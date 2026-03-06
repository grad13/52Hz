use chrono::{Local, Timelike};
use rusqlite::Connection;
use std::path::PathBuf;
use tauri::Emitter;

// ── Data types ──────────────────────────────────────────────

#[derive(serde::Serialize, Clone, Debug)]
pub struct PresenceMessage {
    pub name: String,
    pub message: String,
}

struct ChatMessage {
    name: String,
    message: String,
}

// ── Simple xorshift64 PRNG ──────────────────────────────────

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn range(&mut self, n: u32) -> u32 {
        (self.next() % n as u64) as u32
    }
}

// ── DB queries ──────────────────────────────────────────────

fn load_messages(conn: &Connection, hour: u32) -> Vec<ChatMessage> {
    // Handle wrap-around hours (e.g. 22→3)
    let mut stmt = conn
        .prepare_cached(
            "SELECT name, message FROM chat
             WHERE category = 'during'
               AND (
                 (hour_start < hour_end AND ? >= hour_start AND ? < hour_end)
                 OR (hour_start >= hour_end AND (? >= hour_start OR ? < hour_end))
               )",
        )
        .expect("prepare chat query");

    stmt.query_map([hour, hour, hour, hour], |row| {
        Ok(ChatMessage {
            name: row.get(0)?,
            message: row.get(1)?,
        })
    })
    .expect("query chat")
    .filter_map(|r| r.ok())
    .collect()
}

fn load_density(conn: &Connection, hour: u32) -> f64 {
    conn.query_row(
        "SELECT ratio FROM hourly_density WHERE hour = ?",
        [hour],
        |row| row.get(0),
    )
    .unwrap_or(1.0)
}

// ── Time helpers ────────────────────────────────────────────

const BASE_INTERVAL: f64 = 60.0; // seconds

fn interval_from_ratio(ratio: f64) -> u32 {
    (BASE_INTERVAL / ratio).round().max(20.0) as u32
}

// ── Scheduler loop ──────────────────────────────────────────

pub fn spawn(app: tauri::AppHandle, db_path: PathBuf) {
    tauri::async_runtime::spawn(async move {
        let conn = Connection::open(&db_path).expect("failed to open chat.db");
        let mut rng = Rng::new(Local::now().timestamp() as u64);

        // Wait for webview to load before first message
        tokio::time::sleep(std::time::Duration::from_secs(8)).await;

        loop {
            let hour = Local::now().hour();
            let messages = load_messages(&conn, hour);

            if !messages.is_empty() {
                let idx = rng.range(messages.len() as u32) as usize;
                let msg = &messages[idx];
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] presence: {} — {}", msg.name, msg.message);
                }
                let _ = app.emit(
                    "presence-message",
                    PresenceMessage {
                        name: msg.name.clone(),
                        message: msg.message.clone(),
                    },
                );
            }

            // Interval based on hourly density from DB
            let ratio = load_density(&conn, hour);
            let base = interval_from_ratio(ratio);
            // ±30% randomisation
            let lo = (base as f64 * 0.7) as u32;
            let spread = (base as f64 * 0.6) as u32 + 1;
            let delay = lo + rng.range(spread);
            tokio::time::sleep(std::time::Duration::from_secs(delay as u64)).await;
        }
    });
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE chat (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                message TEXT NOT NULL,
                category TEXT NOT NULL,
                hour_start INTEGER NOT NULL,
                hour_end INTEGER NOT NULL
            );
            CREATE TABLE hourly_density (
                hour INTEGER PRIMARY KEY,
                ratio REAL NOT NULL
            );

            -- Test messages: 朝活 [5, 8)
            INSERT INTO chat (name, message, category, hour_start, hour_end)
            VALUES ('朝活エンジニア', '朝だ、コード書くぞ', 'during', 5, 8);
            INSERT INTO chat (name, message, category, hour_start, hour_end)
            VALUES ('朝活エンジニア', 'コーヒーうまい', 'during', 5, 8);
            INSERT INTO chat (name, message, category, hour_start, hour_end)
            VALUES ('朝活エンジニア', 'おはよう', 'enter', 5, 8);

            -- Test messages: 夜型 [22, 3) wrap-around
            INSERT INTO chat (name, message, category, hour_start, hour_end)
            VALUES ('深夜勉強勢', '静かな夜に集中', 'during', 22, 3);
            INSERT INTO chat (name, message, category, hour_start, hour_end)
            VALUES ('深夜勉強勢', '眠いけどもう少し', 'during', 22, 3);

            -- Test messages: all-day [0, 24)
            INSERT INTO chat (name, message, category, hour_start, hour_end)
            VALUES ('受験生A', 'がんばるぞ', 'during', 6, 23);

            -- Hourly density
            INSERT INTO hourly_density VALUES (0, 1.67);
            INSERT INTO hourly_density VALUES (1, 1.69);
            INSERT INTO hourly_density VALUES (5, 1.32);
            INSERT INTO hourly_density VALUES (12, 0.47);
            INSERT INTO hourly_density VALUES (20, 0.27);
            INSERT INTO hourly_density VALUES (22, 0.86);
            ",
        )
        .unwrap();
        conn
    }

    #[test]
    fn load_messages_normal_range() {
        let conn = setup_test_db();
        // Hour 6: 朝活 [5,8) active, 受験生 [6,23) active
        let msgs = load_messages(&conn, 6);
        assert_eq!(msgs.len(), 3); // 2 朝活 during + 1 受験生 during
        let names: Vec<&str> = msgs.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"朝活エンジニア"));
        assert!(names.contains(&"受験生A"));
    }

    #[test]
    fn load_messages_outside_range() {
        let conn = setup_test_db();
        // Hour 4: nobody active (朝活 starts at 5, 夜型 ends at 3)
        let msgs = load_messages(&conn, 4);
        assert!(msgs.is_empty());
    }

    #[test]
    fn load_messages_wrap_around() {
        let conn = setup_test_db();
        // Hour 23: 夜型 [22,3) active
        let msgs = load_messages(&conn, 23);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].name, "深夜勉強勢");

        // Hour 1: 夜型 [22,3) still active
        let msgs = load_messages(&conn, 1);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].name, "深夜勉強勢");
    }

    #[test]
    fn load_messages_only_during_category() {
        let conn = setup_test_db();
        // Hour 6: should NOT include 'enter' messages
        let msgs = load_messages(&conn, 6);
        // 朝活 has 2 during + 1 enter, but only during should be returned
        for m in &msgs {
            // enter message "おはよう" should not appear
            assert_ne!(m.message, "おはよう");
        }
    }

    #[test]
    fn load_density_known_hour() {
        let conn = setup_test_db();
        let ratio = load_density(&conn, 1);
        assert!((ratio - 1.69).abs() < 0.001);
    }

    #[test]
    fn load_density_missing_hour_returns_default() {
        let conn = setup_test_db();
        // Hour 10 not in test DB
        let ratio = load_density(&conn, 10);
        assert!((ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn interval_from_ratio_peak() {
        // Peak: 60/1.69 ≈ 36
        let iv = interval_from_ratio(1.69);
        assert!(iv >= 30 && iv <= 40, "peak interval={iv}");
    }

    #[test]
    fn interval_from_ratio_quiet() {
        // Quiet: 60/0.27 ≈ 222
        let iv = interval_from_ratio(0.27);
        assert!(iv >= 200 && iv <= 230, "quiet interval={iv}");
    }

    #[test]
    fn interval_minimum_20s() {
        // Very high ratio should still be >= 20
        let iv = interval_from_ratio(100.0);
        assert!(iv >= 20);
    }
}
