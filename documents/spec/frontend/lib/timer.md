---
updated: 2026-03-16 07:20
checked: -
Deprecated: -
Format: spec-v2.1
Source: frontend/lib/timer.ts
---

# timer.ts spec

## 0. Meta

| Item | Value |
|------|-------|
| Module path | `frontend/lib/timer.ts` |
| Runtime | JS-ESM |
| Responsibility | Timer state type definitions, display utilities, and communication layer with the Tauri backend |
| Dependencies | `@tauri-apps/api/core` (invoke), `@tauri-apps/api/event` (listen) |
| Lines | 103 |

This module aggregates timer-related frontend logic.
State management is handled by the backend (Rust) side, and this module functions as a thin client layer that only performs fetching, operations, and subscriptions.

---

## 1. Contract

### 1.1 Type Definitions

#### `TimerSettings` (interface, exported)

| Field | Type | Description |
|-------|------|-------------|
| `focus_duration_secs` | `number` | Focus phase duration (seconds) |
| `short_break_duration_secs` | `number` | Short break duration (seconds) |
| `long_break_duration_secs` | `number` | Long break duration (seconds) |
| `short_breaks_before_long` | `number` | Number of short breaks before a long break |

#### `TimerPhase` (type alias, exported)

```typescript
"Focus" | "ShortBreak" | "LongBreak"
```

Union type representing the three phases. Serialization-compatible with the backend Rust enum.

#### `TimerState` (interface, exported)

| Field | Type | Description |
|-------|------|-------------|
| `phase` | `TimerPhase` | Current phase |
| `paused` | `boolean` | Whether paused |
| `elapsed_secs` | `number` | Elapsed seconds in the current phase |
| `phase_duration_secs` | `number` | Total seconds for the current phase |
| `short_break_count` | `number` | Cumulative short break count |
| `settings` | `TimerSettings` | Current settings |

### 1.2 Pure Functions

#### `remainingSecs(state: TimerState): number`

Returns the remaining seconds in the current phase.

- Calculation: `Math.max(0, state.phase_duration_secs - state.elapsed_secs)`
- Clamped to a minimum of 0 (never negative)

#### `formatTime(totalSecs: number): string`

Converts seconds to a `"MM:SS"` format string.

- Minutes: `Math.floor(totalSecs / 60)`, zero-padded to 2 digits
- Seconds: `totalSecs % 60`, zero-padded to 2 digits
- Example: `formatTime(65)` => `"01:05"`, `formatTime(0)` => `"00:00"`

### 1.3 Tauri invoke Wrappers (async)

| Function name | Tauri command | Arguments | Return value | Description |
|--------------|--------------|-----------|-------------|-------------|
| `getTimerState()` | `get_timer_state` | None | `Promise<TimerState>` | Fetch current state |
| `pauseTimer()` | `pause_timer` | None | `Promise<void>` | Pause |
| `resumeTimer()` | `resume_timer` | None | `Promise<void>` | Resume |
| `togglePause()` | `toggle_pause` | None | `Promise<boolean>` | Toggle. Returns the new paused state |
| `skipBreak()` | `skip_break` | None | `Promise<void>` | Skip break and return to Focus |
| `updateSettings(settings)` | `update_settings` | `{ settings: TimerSettings }` | `Promise<void>` | Update settings |
| `closeBreakOverlay()` | `close_break_overlay` | None | `Promise<void>` | Close the break overlay |
| `quitApp()` | `quit_app` | None | `Promise<void>` | Quit the app |
| `getTodaySessions()` | `get_today_sessions` | None | `Promise<number>` | Fetch today's completed session count |
| `acceptBreak()` | `accept_break` | None | `Promise<void>` | Accept break start |
| `extendFocus(secs)` | `extend_focus` | `{ secs: number }` | `Promise<void>` | Extend the focus phase by the specified seconds |
| `skipBreakFromFocus()` | `skip_break_from_focus` | None | `Promise<void>` | Skip break from the focus phase |
| `resetTimer()` | `reset_timer` | None | `Promise<void>` | Reset the timer to initial state and return to the Focus phase |

### 1.4 Event Listeners

| Function name | Event name | Callback type | Return value |
|--------------|-----------|--------------|-------------|
| `onTimerTick(cb)` | `timer-tick` | `(state: TimerState) => void` | `Promise<UnlistenFn>` |
| `onPhaseChanged(cb)` | `phase-changed` | `(state: TimerState) => void` | `Promise<UnlistenFn>` |
| `onBreakStart(cb)` | `break-start` | `(state: TimerState) => void` | `Promise<UnlistenFn>` |
| `onBreakEnd(cb)` | `break-end` | `() => void` | `Promise<UnlistenFn>` |

All listeners return the `listen()` return value (`Promise<UnlistenFn>`) as-is.
The caller can `await` and then call `unlisten()` to unsubscribe.

---

## 2. State

This module itself holds no state (stateless).

State is held by the backend (Rust `timer.rs`) as `TimerModel`.
The frontend accesses state through the following means:

| Method | Direction | Purpose |
|--------|-----------|---------|
| `getTimerState()` | pull (frontend -> backend) | Initialization and explicit state retrieval |
| `onTimerTick()` | push (backend -> frontend) | Automatic updates every second |
| `onPhaseChanged()` | push (backend -> frontend) | Phase transition notification |
| `onBreakStart()` | push (backend -> frontend) | Break start notification |
| `onBreakEnd()` | push (backend -> frontend) | Break end notification |

### TimerPhase Transition Diagram

```
Focus --[elapsed >= duration]--> ShortBreak or LongBreak
ShortBreak --[elapsed >= duration]--> Focus
LongBreak --[elapsed >= duration]--> Focus (short_break_count reset)
```

- Transitions to ShortBreak when `short_break_count < short_breaks_before_long`
- Transitions to LongBreak when `short_break_count >= short_breaks_before_long`

---

## 3. Logic

### 3.1 remainingSecs

```
Input: TimerState
Output: number (>= 0)
Logic: max(0, phase_duration_secs - elapsed_secs)
```

- Returns 0 even if `elapsed_secs` exceeds `phase_duration_secs` (safety clamp)
- Pure function. No side effects

### 3.2 formatTime

```
Input: number (totalSecs, assumed integer)
Output: string ("MM:SS")
Logic:
  mins = floor(totalSecs / 60)
  secs = totalSecs % 60
  Join with zero-padded 2 digits
```

- For 60+ minutes, displayed as-is (e.g., `formatTime(3661)` => `"61:01"`)
- Negative numbers are not handled (caller is expected to pass 0-clamped values via `remainingSecs`)
- Pure function. No side effects

---

## 4. Side Effects

### 4.1 Tauri invoke (IPC)

All invoke wrappers use `invoke()` from `@tauri-apps/api/core` to call Tauri backend Rust commands.

| Function | Rust command | Direction | Side effect |
|----------|-------------|-----------|-------------|
| `getTimerState` | `get_timer_state` | read | None (state read only) |
| `pauseTimer` | `pause_timer` | write | Sets backend state to paused=true |
| `resumeTimer` | `resume_timer` | write | Sets backend state to paused=false |
| `togglePause` | `toggle_pause` | write | Toggles paused |
| `skipBreak` | `skip_break` | write | Terminates break and transitions to Focus |
| `updateSettings` | `update_settings` | write | Updates settings, immediately reflects current phase duration |
| `closeBreakOverlay` | `close_break_overlay` | write | Closes the overlay window |
| `quitApp` | `quit_app` | write | Terminates the app process |
| `getTodaySessions` | `get_today_sessions` | read | None (count read only) |
| `acceptBreak` | `accept_break` | write | Notifies break acceptance to backend |
| `extendFocus` | `extend_focus` | write | Extends Focus phase duration |
| `skipBreakFromFocus` | `skip_break_from_focus` | write | Skips break transition from Focus phase |
| `resetTimer` | `reset_timer` | write | Stops timer, resets elapsed_secs, returns phase to Focus |

### 4.2 Tauri listen (Event Subscription)

All listeners use `listen()` from `@tauri-apps/api/event` to subscribe to push events from the backend.

| Event | Payload | Trigger timing |
|-------|---------|---------------|
| `timer-tick` | `TimerState` | Emitted every second by backend's `advance()` |
| `phase-changed` | `TimerState` | When phase changes in `try_transition()` |
| `break-start` | `TimerState` | On Focus -> ShortBreak/LongBreak transition |
| `break-end` | None | On ShortBreak/LongBreak -> Focus transition |

Listener functions return `Promise<UnlistenFn>`. The component's `onDestroy` / cleanup should call unlisten to unsubscribe.

---

## 5. Notes

### Design Intent

- The frontend is responsible only for "display" and "action triggering"; all timer logic (counting, transition decisions) is delegated to the backend
- This design ensures the timer continues running even when the frontend window is closed
- Type definitions (`TimerSettings`, `TimerPhase`, `TimerState`) mirror the backend Rust structs and maintain compatibility via serde JSON serialization

### Testing Considerations

- `remainingSecs` and `formatTime` are pure functions and are easy to unit test
- invoke wrappers and listeners depend on Tauri IPC and require mocking or integration tests
- The backend timer logic itself has 69 unit tests in `code/app/tauri/src/timer.rs`

### Caveats

- `formatTime` does not handle negative input. It assumes 0-clamped values are passed via `remainingSecs`
- `formatTime` displays in `"MM:SS"` format even for 60+ minutes (e.g., `"61:01"`). The minute portion may have 3 digits
- Only `onBreakEnd` has no payload (`() => void`). The other 3 listeners receive `TimerState`
- The argument to `updateSettings` is passed to invoke in object format `{ settings: TimerSettings }` (Tauri's named argument format)
