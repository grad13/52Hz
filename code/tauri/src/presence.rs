use chrono::{Datelike, Local, Timelike};
use rusqlite::Connection;
use std::path::PathBuf;
use tauri::{Emitter, Manager};
use tokio::sync::watch;

// ── Data types ──────────────────────────────────────────────

#[derive(serde::Serialize, Clone, Debug)]
pub struct PresenceMessage {
    pub name: String,
    pub message: String,
}

struct TapeMessage {
    id: i64,
    name: String,
    text: String,
    at: i64,
}

// ── Tape position helpers ───────────────────────────────────

/// Convert current local time to tape position (minutes from Monday 00:00).
/// Monday=0, Tuesday=1, ..., Sunday=6.
fn current_tape_position() -> i64 {
    let now = Local::now();
    let dow = now.weekday().num_days_from_monday() as i64; // Mon=0
    dow * 1440 + now.hour() as i64 * 60 + now.minute() as i64
}

/// Total minutes in one week (tape length).
const TAPE_LENGTH: i64 = 10080; // 7 * 1440

// ── DB queries ──────────────────────────────────────────────

/// Get the next message at or after the given tape position.
/// If none found (past end of tape), returns None.
fn next_message(conn: &Connection, at: i64) -> Option<TapeMessage> {
    conn.prepare_cached(
        "SELECT m.id, u.name, m.text, m.at
         FROM message m JOIN user u ON m.user_id = u.id
         WHERE m.at >= ?
         ORDER BY m.at, m.id
         LIMIT 1",
    )
    .expect("prepare next_message query")
    .query_row([at], |row| {
        Ok(TapeMessage {
            id: row.get(0)?,
            name: row.get(1)?,
            text: row.get(2)?,
            at: row.get(3)?,
        })
    })
    .ok()
}

/// Get the next message after a specific id at the same or later tape position.
/// Used to advance through multiple messages at the same `at`.
fn next_message_after(conn: &Connection, at: i64, after_id: i64) -> Option<TapeMessage> {
    conn.prepare_cached(
        "SELECT m.id, u.name, m.text, m.at
         FROM message m JOIN user u ON m.user_id = u.id
         WHERE m.at >= ? AND m.id > ?
         ORDER BY m.at, m.id
         LIMIT 1",
    )
    .expect("prepare next_message_after query")
    .query_row([at, after_id], |row| {
        Ok(TapeMessage {
            id: row.get(0)?,
            name: row.get(1)?,
            text: row.get(2)?,
            at: row.get(3)?,
        })
    })
    .ok()
}

/// Get the first message on the tape (for wrap-around).
fn first_message(conn: &Connection) -> Option<TapeMessage> {
    next_message(conn, 0)
}

// ── Cassette directory management ───────────────────────────

/// Ensure ~/Documents/52Hz/ exists and copy default.hz if needed.
pub fn ensure_cassette_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = dirs::document_dir()
        .expect("document_dir")
        .join("52Hz");
    std::fs::create_dir_all(&dir).ok();

    let default_dst = dir.join("default.hz");
    if !default_dst.exists() {
        if let Ok(resource_dir) = app.path().resource_dir() {
            let bundled: PathBuf = resource_dir.join("default.hz");
            if bundled.exists() {
                std::fs::copy(&bundled, &default_dst).ok();
            }
        }
    }
    dir
}

/// List available cassettes in the cassette directory.
/// Returns Vec of (path, title).
pub fn list_cassettes(cassette_dir: &std::path::Path) -> Vec<(PathBuf, String)> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(cassette_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "hz").unwrap_or(false) {
                let title = cassette_title(&path).unwrap_or_else(|| {
                    path.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                });
                result.push((path, title));
            }
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
    result
}

/// Read the title from a .hz file.
fn cassette_title(path: &std::path::Path) -> Option<String> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    conn.query_row("SELECT title FROM cassette", [], |row| row.get(0)).ok()
}

// ── Sleep with cassette switch check ────────────────────────

/// Sleep for `secs` seconds, but wake up if cassette is switched.
/// Returns `true` if a cassette switch happened.
async fn wait_for_or_switch(
    rx: &mut watch::Receiver<PathBuf>,
    current_path: &mut PathBuf,
    conn: &mut Connection,
    secs: u64,
) -> bool {
    let mut remaining = secs;
    while remaining > 0 {
        let chunk = remaining.min(5); // check every 5 seconds
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(chunk)) => {
                remaining = remaining.saturating_sub(chunk);
            }
            _ = rx.changed() => {
                let new_path = rx.borrow().clone();
                if new_path != *current_path {
                    *current_path = new_path;
                    if let Ok(new_conn) = Connection::open_with_flags(
                        &*current_path,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                    ) {
                        *conn = new_conn;
                        if cfg!(debug_assertions) {
                            eprintln!("[52Hz] presence: cassette switched to {:?}", current_path);
                        }
                    }
                    return true;
                }
            }
        }
    }
    false
}

// ── Scheduler loop ──────────────────────────────────────────

pub fn spawn(app: tauri::AppHandle, hz_path: PathBuf, mut rx: watch::Receiver<PathBuf>) {
    tauri::async_runtime::spawn(async move {
        // Wait for webview to load before first message
        tokio::time::sleep(std::time::Duration::from_secs(8)).await;

        let mut current_path = hz_path;
        let mut conn = Connection::open_with_flags(
            &current_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .expect("failed to open .hz cassette");

        // Track the last emitted message id to advance through the tape
        let mut last_emitted_id: i64 = 0;

        loop {
            let tape_pos = current_tape_position();

            // Find next message: after current tape position,
            // or if multiple at same `at`, advance by id
            let msg = next_message_after(&conn, tape_pos, last_emitted_id)
                .or_else(|| next_message(&conn, tape_pos))
                .or_else(|| first_message(&conn)); // wrap around

            if let Some(msg) = msg {
                // Calculate wait time
                let wait_mins = if msg.at >= tape_pos {
                    msg.at - tape_pos
                } else {
                    (TAPE_LENGTH - tape_pos) + msg.at
                };

                if wait_mins > 0 {
                    let wait_secs = wait_mins as u64 * 60;
                    if wait_for_or_switch(&mut rx, &mut current_path, &mut conn, wait_secs).await {
                        last_emitted_id = 0;
                        continue;
                    }
                }

                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] presence: {} — {}", msg.name, msg.text);
                }
                let _ = app.emit(
                    "presence-message",
                    PresenceMessage {
                        name: msg.name.clone(),
                        message: msg.text.clone(),
                    },
                );
                last_emitted_id = msg.id;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        }
    });
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_hz() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE cassette (version REAL NOT NULL, title TEXT NOT NULL);
             CREATE TABLE user (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
             CREATE TABLE message (
                 id INTEGER PRIMARY KEY,
                 user_id INTEGER NOT NULL REFERENCES user(id),
                 at INTEGER NOT NULL,
                 text TEXT NOT NULL
             );
             CREATE INDEX idx_message_at ON message(at);

             INSERT INTO cassette VALUES (1.0, 'Test Cassette');
             INSERT INTO user VALUES (1, 'Alice');
             INSERT INTO user VALUES (2, 'Bob');

             -- Monday 08:30 (at=510)
             INSERT INTO message VALUES (1, 1, 510, 'Good morning');
             -- Monday 08:30 (same time, different user)
             INSERT INTO message VALUES (2, 2, 510, 'Morning!');
             -- Monday 09:00 (at=540)
             INSERT INTO message VALUES (3, 1, 540, 'Coffee time');
             -- Friday 18:00 (at=6360)
             INSERT INTO message VALUES (4, 2, 6360, 'Weekend soon');
             -- Sunday 23:30 (at=10050)
             INSERT INTO message VALUES (5, 1, 10050, 'Good night');
            ",
        )
        .unwrap();
        conn
    }

    #[test]
    fn next_message_finds_closest() {
        let conn = setup_test_hz();
        let msg = next_message(&conn, 500).unwrap();
        assert_eq!(msg.at, 510);
        assert_eq!(msg.id, 1); // First message at 510 by id order
    }

    #[test]
    fn next_message_exact_match() {
        let conn = setup_test_hz();
        let msg = next_message(&conn, 510).unwrap();
        assert_eq!(msg.at, 510);
    }

    #[test]
    fn next_message_after_advances_through_same_at() {
        let conn = setup_test_hz();
        // First at 510 is id=1 (Alice), after id=1 should get id=2 (Bob)
        let msg = next_message_after(&conn, 510, 1).unwrap();
        assert_eq!(msg.id, 2);
        assert_eq!(msg.at, 510);
        assert_eq!(msg.name, "Bob");
    }

    #[test]
    fn next_message_after_jumps_to_next_at() {
        let conn = setup_test_hz();
        // After id=2 (last at 510), should jump to id=3 at 540
        let msg = next_message_after(&conn, 510, 2).unwrap();
        assert_eq!(msg.id, 3);
        assert_eq!(msg.at, 540);
    }

    #[test]
    fn next_message_skips_past() {
        let conn = setup_test_hz();
        let msg = next_message(&conn, 511).unwrap();
        assert_eq!(msg.at, 540);
        assert_eq!(msg.text, "Coffee time");
    }

    #[test]
    fn next_message_past_end_returns_none() {
        let conn = setup_test_hz();
        assert!(next_message(&conn, 10051).is_none());
    }

    #[test]
    fn first_message_wraps_around() {
        let conn = setup_test_hz();
        let msg = first_message(&conn).unwrap();
        assert_eq!(msg.at, 510);
    }

    #[test]
    fn tape_position_range() {
        let pos = current_tape_position();
        assert!(pos >= 0 && pos < TAPE_LENGTH);
    }

    #[test]
    fn cassette_title_reads_correctly() {
        // Test with in-memory isn't practical for file-based cassette_title,
        // so we test the query logic directly
        let conn = setup_test_hz();
        let title: String = conn
            .query_row("SELECT title FROM cassette", [], |row| row.get(0))
            .unwrap();
        assert_eq!(title, "Test Cassette");
    }
}
