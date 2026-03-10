---
Created: 2026-02-26
Updated: 2026-03-08
Verified: -
Retired: -
Format: spec-v2.1
Source: tauri/src/commands.rs
---

# spec: commands (Tauri IPC Command Layer)

## 0. Meta

| Item | Value |
|------|-------|
| Runtime | Rust (Tauri v2) |
| Test Type | Integration (process launch + IPC invocation) |
| Target File | `code/tauri/src/commands.rs` (245 lines) |
| Responsibility | A thin command layer that receives IPC calls from the frontend and performs timer state manipulation, UI control, and app termination |

## 1. Contract

### 1.1 Helper Functions

#### `do_toggle_pause`

```
pub(crate) async fn do_toggle_pause(
    app: &tauri::AppHandle,
    state: &SharedTimerState,
) -> bool
```

- **Precondition**: `state` is a valid `Arc<Mutex<TimerState>>`
- **Postcondition**: `state.paused` is toggled. `timer-tick` event is emitted
- **Return value**: The new `paused` value (`true` = paused)

### 1.2 Tauri Commands

All commands share:
- Visibility: `pub(crate)` (accessible only via Tauri IPC)
- Error type: `Result<T, String>` (Tauri command convention)

#### `get_timer_state`

```
#[tauri::command]
async fn get_timer_state(state: State<SharedTimerState>) -> Result<TimerState, String>
```

- **Input**: None (state is injected via Tauri DI)
- **Output**: A clone of the current `TimerState`
- **Side effects**: None (read-only)

#### `pause_timer`

```
#[tauri::command]
async fn pause_timer(app: AppHandle, state: State<SharedTimerState>) -> Result<(), String>
```

- **Postcondition**: `state.paused == true`
- **Idempotency**: Safe to call even if already paused

#### `resume_timer`

```
#[tauri::command]
async fn resume_timer(app: AppHandle, state: State<SharedTimerState>) -> Result<(), String>
```

- **Postcondition**: `state.paused == false`
- **Idempotency**: Safe to call even if already running

#### `toggle_pause`

```
#[tauri::command]
async fn toggle_pause(app: AppHandle, state: State<SharedTimerState>) -> Result<bool, String>
```

- **Delegates to**: `do_toggle_pause`
- **Side effects**: Emits `timer-tick` event through delegation to `do_toggle_pause`
- **Return value**: The new `paused` value

#### `skip_break`

```
#[tauri::command]
async fn skip_break(app: AppHandle, state: State<SharedTimerState>) -> Result<(), String>
```

- **Precondition**: Only effective when `state.phase` is `ShortBreak` or `LongBreak`
- **Postcondition** (during Break phase): `phase` transitions to `Focus`, `elapsed_secs` resets to 0, `break-end` + `phase-changed` events are emitted
- **Postcondition** (during Focus phase): Nothing happens (`events` is empty)

#### `update_settings`

```
#[tauri::command]
async fn update_settings(
    app: AppHandle,
    state: State<SharedTimerState>,
    settings: TimerSettings,
) -> Result<(), String>
```

- **Input**: `TimerSettings { focus_duration_secs, short_break_duration_secs, long_break_duration_secs, short_breaks_before_long }`
- **Postcondition**: Settings are applied immediately (including `phase_duration_secs` for the current phase), `timer-tick` event emitted

#### `accept_break`

```
#[tauri::command]
async fn accept_break(app: AppHandle, state: State<SharedTimerState>) -> Result<(), String>
```

- **Postcondition**: Accepts the break after FocusDone. Calls `s.accept_break()` and emits `phase-changed` and `break-start` events based on the returned `PhaseEvent`. Emits `timer-tick` event
- **Events emitted**: `PhaseEvent::PhaseChanged` → `app.emit("phase-changed", state_clone)`, `PhaseEvent::BreakStart` → `app.emit("break-start", state_clone)`, always `app.emit("timer-tick", state_clone)`
- **Session recording**: Calls `increment_today_sessions(&app)` to increment the `sessions_YYYY-MM-DD` key in the `settings.json` store
- **Return value**: `Result<(), String>`

#### `extend_focus(secs: u64)`

```
#[tauri::command]
async fn extend_focus(app: AppHandle, state: State<SharedTimerState>, secs: u64) -> Result<(), String>
```

- **Input**: `secs: u64` — number of seconds to extend focus
- **Postcondition**: Calls `s.extend_focus(secs)` to extend focus time. Emits `timer-tick` event
- **Return value**: `Result<(), String>`

#### `skip_break_from_focus`

```
#[tauri::command]
async fn skip_break_from_focus(app: AppHandle, state: State<SharedTimerState>) -> Result<(), String>
```

- **Postcondition**: Skips break from FocusDone. Calls `s.skip_break_from_focus()` and emits `phase-changed` event only if the returned `PhaseEvent` contains `PhaseChanged`. Always emits `timer-tick` event
- **Events emitted**: `PhaseEvent::PhaseChanged` → `app.emit("phase-changed", state_clone)`, always `app.emit("timer-tick", state_clone)`
- **Session recording**: Calls `increment_today_sessions(&app)` to increment the `sessions_YYYY-MM-DD` key in the `settings.json` store
- **Return value**: `Result<(), String>`

#### `open_break_overlay`

```
#[tauri::command]
async fn open_break_overlay(app: AppHandle) -> Result<(), String>
```

- **Postcondition**: Break overlay window is created
- **Execution thread**: Main thread (`run_on_main_thread`)
- **Error**: `Err(String)` on main thread dispatch failure

#### `quit_app`

```
#[tauri::command]
fn quit_app()
```

- **Postcondition**: Process terminates immediately with exit code 0
- **Note**: Synchronous function. No cleanup. Logs to stderr only in debug builds

#### `close_break_overlay`

```
#[tauri::command]
async fn close_break_overlay(app: AppHandle) -> Result<(), String>
```

- **Postcondition**: Window with label `"break-overlay"` is closed (no-op if it doesn't exist)
- **Execution thread**: Main thread (`run_on_main_thread`)

#### `reset_timer`

```
#[tauri::command]
async fn reset_timer(app: AppHandle, state: State<SharedTimerState>) -> Result<(), String>
```

- **Parameters**: `app: AppHandle`, `state: State<SharedTimerState>`
- **Flow**: lock → `s.reset()` → emit `timer-tick`
- **Side effects**: Resets timer to paused Focus phase state. Emits `timer-tick` event

#### `get_today_sessions`

```
#[tauri::command]
pub(crate) async fn get_today_sessions(app: tauri::AppHandle) -> Result<u64, String>
```

- **Input**: None (app is injected via Tauri DI)
- **Output**: Number of sessions completed today (`u64`)
- **Side effects**: None (read-only)
- **Store key**: Reads the `sessions_YYYY-MM-DD` key (generated from `chrono::Local::now()`) from the `settings.json` store
- **Fallback**: Returns `0` if store open fails or key doesn't exist (does not return an error)

### 1.3 Type Definitions

#### `CassetteInfo`

```rust
#[derive(serde::Serialize, Clone)]
pub(crate) struct CassetteInfo {
    pub path: String,    // Absolute path of the .hz file
    pub title: String,   // Cassette title
}
```

### 1.4 Cassette-Related Commands

#### `list_cassettes`

```
#[tauri::command]
async fn list_cassettes(app: AppHandle) -> Result<Vec<CassetteInfo>, String>
```

- **Input**: None (app is injected via Tauri DI)
- **Output**: List of `.hz` files in the cassette directory. Each entry includes path and title
- **Side effects**: Ensures cassette directory via `presence::ensure_cassette_dir()` and retrieves list via `presence::list_cassettes()`
- **Flow**: `ensure_cassette_dir(app)` → `list_cassettes(&dir)` → Convert `Vec<(PathBuf, String)>` to `Vec<CassetteInfo>`

#### `switch_cassette`

```
#[tauri::command]
async fn switch_cassette(app: AppHandle, path: String) -> Result<(), String>
```

- **Input**: `path: String` — Path of the `.hz` file to switch to
- **Postcondition**: New cassette path is sent through the `CassetteSwitcher` watch channel
- **Flow**: `app.state::<CassetteSwitcher>()` → `tx.lock()` → `tx.send(PathBuf::from(path))`
- **Error**: `Err(String)` on Mutex lock failure or channel send failure

#### `open_cassette_folder`

```
#[tauri::command]
async fn open_cassette_folder(app: AppHandle) -> Result<(), String>
```

- **Input**: None (app is injected via Tauri DI)
- **Postcondition**: `~/Documents/52Hz/` is opened in Finder
- **Flow**: `ensure_cassette_dir(app)` → `std::process::Command::new("open").arg(&dir).spawn()`
- **Error**: `Err(String)` on `open` command spawn failure

## 2. State

### 2.1 Shared State

```rust
type SharedTimerState = Arc<Mutex<TimerState>>;
```

All commands receive the same `SharedTimerState` via Tauri's State DI.
Exclusive access pattern: lock via async Mutex (`tokio::sync::Mutex` equivalent) → operate → drop.

### 2.2 TimerState Structure

```rust
pub struct TimerState {
    pub phase: TimerPhase,           // Focus | ShortBreak | LongBreak
    pub paused: bool,
    pub elapsed_secs: u64,
    pub phase_duration_secs: u64,
    pub short_break_count: u32,
    pub settings: TimerSettings,
}
```

### 2.3 State Transitions (those involving commands.rs)

| Operation | Changed Fields |
|-----------|----------------|
| `pause_timer` | `paused = true` |
| `resume_timer` | `paused = false` |
| `toggle_pause` | `paused = !paused` |
| `skip_break` | `phase = Focus`, `elapsed_secs = 0`, `phase_duration_secs = settings.focus_duration_secs` (only during Break) |
| `update_settings` | Entire `settings` + `phase_duration_secs` (recalculated based on current phase) |
| `accept_break` | FocusDone → Transitions to ShortBreak/LongBreak (delegated to `s.accept_break()`). Increments `sessions_YYYY-MM-DD` in store |
| `extend_focus` | Extends focus duration (delegated to `s.extend_focus(secs)`) |
| `skip_break_from_focus` | FocusDone → Transitions to Focus (delegated to `s.skip_break_from_focus()`). Increments `sessions_YYYY-MM-DD` in store |
| `reset_timer` | Resets to paused Focus phase state (delegated to `s.reset()`) |
| `get_today_sessions` | No change (read-only) |
| `list_cassettes` | No change (read-only) |
| `switch_cassette` | Sends new path to `CassetteSwitcher` watch channel (does not affect TimerState) |
| `open_cassette_folder` | No change (external process launch only) |

## 3. Logic

### 3.1 do_toggle_pause Flow

```
1. state.lock().await
2. paused = !paused
3. Capture paused value and state clone
4. Release lock (drop)
5. app.emit("timer-tick", state_clone)
6. return paused
```

**Note**: Event emission occurs after lock release. This avoids deadlocks.

### 3.2 skip_break Flow

```
1. state.lock().await
2. s.skip_break() -> Vec<PhaseEvent>
3. If events is not empty:
   a. app.emit("break-end", ())
   b. app.emit("phase-changed", s.clone())
4. return Ok(())
```

**Note**: The contents of `skip_break()`'s return value (`Vec<PhaseEvent>`) are not inspected; only whether it's empty determines the branching.

**Note**: `skip_break` does not emit `timer-tick`. This is because the `phase-changed` payload already contains `TimerState`, making it redundant, and since `break-end` handles overlay closing, a `timer-tick` UI update trigger is unnecessary. The asymmetry with `skip_break_from_focus` (which does emit `timer-tick`) is due to the overlay context difference.

### 3.3 Overlay Operations Thread Model

```
async command received (Tokio runtime)
  -> app.run_on_main_thread(move || { ... })
     -> NSWindow operations on macOS main thread
```

Errors within the `run_on_main_thread` closure are silently ignored with `let _ =`. Only the outer dispatch error is propagated.

### 3.4 update_settings Flow

```
1. state.lock().await
2. s.apply_settings(settings)  — Immediately updates settings and current phase duration
3. app.emit("timer-tick", s.clone())  — Emits while holding the lock
4. return Ok(())  — s goes out of scope → lock released
```

**Note**: Unlike `do_toggle_pause` which emits after releasing the lock, `update_settings` emits while holding the lock. Since Tauri's `emit` delivers asynchronously via channels, there is no practical deadlock risk.

## 4. Side Effects

### 4.1 Event Emission

| Event Name | Source | Payload | Receiver |
|-----------|--------|---------|----------|
| `timer-tick` | `toggle_pause` → `do_toggle_pause` (delegation), `update_settings` (while holding lock), `accept_break`, `extend_focus`, `skip_break_from_focus`, `reset_timer` | `TimerState` | Frontend (UI update) |
| `break-end` | `skip_break` | `()` | Frontend (close overlay) |
| `phase-changed` | `skip_break`, `accept_break`, `skip_break_from_focus` | `TimerState` | Frontend (phase display update) |
| `break-start` | `accept_break` | `TimerState` | Frontend (break start notification) |

### 4.2 Store Writes (Daily Session Recording)

| Function | Store File | Key Format | Effect |
|----------|-----------|---------|--------|
| `accept_break` | `settings.json` | `sessions_YYYY-MM-DD` | Increments today's count by +1 (starts at 1 if not present) |
| `skip_break_from_focus` | `settings.json` | `sessions_YYYY-MM-DD` | Increments today's count by +1 (starts at 1 if not present) |

Helper function `increment_today_sessions(app: &tauri::AppHandle)` is called from the above 2 commands.

### 4.3 Window Operations

| Function | Effect |
|----------|--------|
| `open_break_overlay` | Creates new window via `overlay::create_break_overlay` |
| `close_break_overlay` | Calls `close()` on the `"break-overlay"` window |

### 4.4 Cassette Operations

| Function | Effect |
|----------|--------|
| `list_cassettes` | File system read via `presence::ensure_cassette_dir()` + `presence::list_cassettes()` |
| `switch_cassette` | Sends `PathBuf` to `CassetteSwitcher` watch channel → presence scheduler switches to new cassette |
| `open_cassette_folder` | Opens cassette directory in Finder via `std::process::Command::new("open")` |

### 4.5 Process Termination

| Function | Effect |
|----------|--------|
| `quit_app` | `std::process::exit(0)` (immediate termination, destructors not run) |

## 5. Notes

### 5.1 Design Decisions

- **Thin command layer**: commands.rs itself holds no business logic; it strictly delegates to `timer.rs` method calls + event emission + UI synchronization
- **Separation of `do_toggle_pause`**: Extracted as shared logic callable from both the command (`toggle_pause`) and the tray menu handler. Application of the DRY principle
- **Error suppression policy**: Event emission (`emit`) failures are ignored with `let _ =`. Safe even when no listeners exist (e.g., right after startup)

### 5.2 Test Strategy

- **Unit tests**: No inline tests in commands.rs. Core logic is already covered in `timer.rs` (69 tests)
- **Integration tests**: Process launch + stderr log inspection in `tests/integration/app_lifecycle.rs`
- **Command layer test gap**: No mechanism currently exists to directly test IPC command inputs/outputs. Adding command tests using Tauri's `tauri::test` utilities is a future candidate

### 5.3 Known Limitations

1. `quit_app` uses `std::process::exit(0)`, so Rust destructors (`Drop`) are not executed. There is a possibility of resource leaks (file locks, etc.), but this is not a problem for the current use case
2. `open_break_overlay` does not prevent duplicate window creation at the command layer. It assumes `overlay::create_break_overlay` handles this
3. `close_break_overlay` silently succeeds when the window doesn't exist (idempotent)
