---
updated: 2026-03-16 07:20
checked: -
Retired: -
Format: spec-v2.1
Source: tauri/src/presence.rs
---

# presence.rs spec

Runtime: Rust

## 0. Meta

| Item | Value |
|---|---|
| File | `code/app/tauri/src/presence.rs` |
| Responsibility | A scheduler that reads messages from .hz cassettes (SQLite) along a weekly time-tape and emits them to the frontend |
| Lines | ~343 (main ~231, tests ~112) |
| Visibility | `pub` -- `PresenceMessage`, `ensure_cassette_dir`, `list_cassettes`, `spawn` |
| Dependencies | `chrono`, `rusqlite`, `tauri`, `tokio::sync::watch`, `dirs` |
| Dependents | `lib.rs` (calls `ensure_cassette_dir` + `spawn` at startup) |

## 1. Contract

### 1.1 Data Types

#### `PresenceMessage` (pub, for frontend)

```rust
#[derive(serde::Serialize, Clone, Debug)]
pub struct PresenceMessage {
    pub name: String,
    pub message: String,
}
```

- Sent to the frontend via `emit("presence-message", PresenceMessage)`

#### `TapeMessage` (internal)

```rust
struct TapeMessage {
    id: i64,
    name: String,
    text: String,
    at: i64,
}
```

- Internal row representation retrieved from `message JOIN user` in the DB

### 1.2 Functions

#### `current_tape_position() -> i64`

- Converts current local time to "minutes elapsed since Monday 00:00"
- Range: `0..10080`
- Monday=0, Tuesday=1, ..., Sunday=6

#### `next_message(conn, at) -> Option<TapeMessage>`

- Returns the first message at or after `at` (`WHERE m.at >= ?`, `ORDER BY m.at, m.id`)
- Returns `None` if past the tape end

#### `next_message_after(conn, at, after_id) -> Option<TapeMessage>`

- Returns the first message at or after `at` with `id > after_id`
- Used for advancing through multiple messages at the same `at`

#### `first_message(conn) -> Option<TapeMessage>`

- Alias for `next_message(conn, 0)`
- Used for wrap-around from tape end

#### `ensure_cassette_dir(app) -> PathBuf`

```rust
pub fn ensure_cassette_dir(app: &tauri::AppHandle) -> PathBuf
```

- Creates `~/Documents/52Hz/` directory (if it doesn't exist)
- Copies `default.hz` from bundle resources if it doesn't exist
- Returns the directory path

#### `list_cassettes(dir) -> Vec<(PathBuf, String)>`

```rust
pub fn list_cassettes(cassette_dir: &std::path::Path) -> Vec<(PathBuf, String)>
```

- Enumerates `.hz` files in the specified directory
- Retrieves `title` from each file's `cassette` table (falls back to filename if retrieval fails)
- Returns sorted by title in ascending order

#### `cassette_title(path) -> Option<String>`

- Opens the `.hz` file (SQLite) in read-only mode and executes `SELECT title FROM cassette`
- Returns the title (`None` if cannot open / table doesn't exist)

#### `wait_for_or_switch(rx, current_path, conn, secs) -> bool`

```rust
async fn wait_for_or_switch(
    rx: &mut watch::Receiver<PathBuf>,
    current_path: &mut PathBuf,
    conn: &mut Connection,
    secs: u64,
) -> bool
```

- Sleeps for `secs` seconds, but checks for cassette switch every 5 seconds
- If cassette was switched: updates `current_path` and `conn`, returns `true`
- If sleep completed: returns `false`

#### `spawn(app, hz_path, rx)`

```rust
pub fn spawn(app: tauri::AppHandle, hz_path: PathBuf, mut rx: watch::Receiver<PathBuf>)
```

- Launches the main scheduler loop via `tauri::async_runtime::spawn`
- Waits 8 seconds initially (for webview loading)

## 2. State

### 2.1 Constants

| Constant | Value | Description |
|---|---|---|
| `TAPE_LENGTH` | `10080` | Minutes in a week (7 * 1440) |

### 2.2 Loop State

| Variable | Type | Description |
|---|---|---|
| `current_path` | `PathBuf` | Path of the currently open cassette |
| `conn` | `Connection` | SQLite connection to the current cassette (read-only) |
| `last_emitted_id` | `i64` | ID of the last emitted message (for advancing through same `at`, initial value 0) |

## 3. Logic

### 3.1 Scheduler Loop

```
spawn -> 8 second wait -> loop:
  tape_pos = current_tape_position()
  msg = next_message_after(tape_pos, last_emitted_id)
      || next_message(tape_pos)
      || first_message()            // Wrap-around

  if msg found:
    wait_mins = msg.at >= tape_pos
                  ? msg.at - tape_pos
                  : (TAPE_LENGTH - tape_pos) + msg.at

    if wait_mins > 0:
      if wait_for_or_switch(wait_mins * 60):
        last_emitted_id = 0         // Cassette switched -> reset
        continue

    emit("presence-message", PresenceMessage)
    last_emitted_id = msg.id
  else:
    sleep(60s)                      // No messages -> retry after 1 minute
```

### 3.2 Message Search Priority

1. `next_message_after(tape_pos, last_emitted_id)` -- Continue from previous
2. `next_message(tape_pos)` -- Fallback (no id condition)
3. `first_message()` -- Wrap-around when past tape end

### 3.3 Wait Time Calculation

- `msg.at >= tape_pos`: Same week -> `msg.at - tape_pos` minutes
- `msg.at < tape_pos`: Cross-week -> `(TAPE_LENGTH - tape_pos) + msg.at` minutes

### 3.4 Cassette Switching

- Receives new cassette path via `tokio::sync::watch` channel
- 5-second interval check loop within `wait_for_or_switch`
- On switch: Reopen DB connection, reset `last_emitted_id`, and continue from loop start

## 4. Side Effects

### 4.1 Event Emission

| Event | Payload | Timing |
|---|---|---|
| `"presence-message"` | `PresenceMessage { name, message }` | Emits message at scheduled time |

### 4.2 File I/O

| Operation | Target | Description |
|---|---|---|
| `create_dir_all` | `~/Documents/52Hz/` | Create cassette directory |
| `std::fs::copy` | `default.hz` | Copy default cassette from bundle resources |
| `read_dir` | `~/Documents/52Hz/` | Retrieve cassette listing |
| `Connection::open_with_flags` | `.hz` files | SQLite read-only connection |

### 4.3 Async Runtime

| Operation | Description |
|---|---|
| `tauri::async_runtime::spawn` | Launch scheduler loop as async task |
| `tokio::time::sleep` | Wait between messages |
| `tokio::select!` | Race between sleep and cassette switch |

## 5. Tests

### 5.1 Unit Tests (`#[cfg(test)]` inline, 9 tests)

| Test | Verification |
|---|---|
| `next_message_finds_closest` | Searching at `at=500` returns the first message at `at=510` (id=1) |
| `next_message_exact_match` | Searching at `at=510` returns the message at `at=510` |
| `next_message_after_advances_through_same_at` | At same `at=510`, after id=1, returns id=2 (Bob) |
| `next_message_after_jumps_to_next_at` | After id=2 (last at 510), jumps to id=3 (at=540) |
| `next_message_skips_past` | `at=511` skips 510 and returns `at=540` |
| `next_message_past_end_returns_none` | Returns `None` for positions past tape end |
| `first_message_wraps_around` | `first_message` returns `at=510` (first message on tape) |
| `tape_position_range` | `current_tape_position()` is within range `0..TAPE_LENGTH` |
| `cassette_title_reads_correctly` | Correctly reads title "Test Cassette" from `cassette` table |

### 5.2 Test Data

| id | user | at | text | Note |
|---|---|---|---|---|
| 1 | Alice | 510 | "Good morning" | Monday 08:30 |
| 2 | Bob | 510 | "Morning!" | Monday 08:30 (same time) |
| 3 | Alice | 540 | "Coffee time" | Monday 09:00 |
| 4 | Bob | 6360 | "Weekend soon" | Friday 18:00 |
| 5 | Alice | 10050 | "Good night" | Sunday 23:30 |

### 5.3 Test Strategy

- DB query functions: Unit testable with in-memory SQLite
- `cassette_title`: File-based, so only query logic is directly tested
- Scheduler loop / async processing: Not under test, manual verification

## 6. Notes

### 6.1 .hz Cassette Format

- SQLite database (extension `.hz`)
- Tables: `cassette(version, title)`, `user(id, name)`, `message(id, user_id, at, text)`
- `at` is minutes elapsed since Monday 00:00 (0--10079)

### 6.2 Design Choices

- **Weekly time-tape model:** Places messages on a fixed one-week timeline, replaying the same schedule every week
- **8-second initial wait:** Hardcoded delay to wait for webview load completion
- **5-second check interval:** Balance between cassette switch responsiveness and CPU load

### 6.3 Error Handling

- DB connection failure: `ensure_cassette_dir`'s copy is ignored with `.ok()`, `spawn`'s initial connection panics with `expect`
- Query failure: Converted to `None` with `.ok()` (treated the same as no message)
- Connection failure on cassette switch: If new connection cannot be opened, continues with old connection (implicit fallback)
