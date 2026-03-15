---
Created: 2026-02-26
Updated: 2026-02-26
Verified: -
Deprecated: -
Format: spec-v2.1
Source: frontend/components/BreakOverlay.svelte
---

# BreakOverlay spec

## 0. Meta

| Source | Runtime |
|--------|---------|
| `frontend/components/BreakOverlay.svelte` | JS-ESM (Svelte 5 component) |

| key | value |
|-----|-------|
| Responsibility | Displays a fullscreen overlay during break phases, presenting phase-specific messages and a countdown timer. Accepts skip actions from the user. |
| Dependencies | `../lib/timer` (type definitions + IPC wrappers), `@tauri-apps/api/window` |

## 1. Contract

### 1.1 Props

None. This component does not receive props.

### 1.2 Public Events

None. No custom events are emitted.

### 1.3 Tauri IPC Command Calls

| Command | Arguments | Return value | Invocation condition |
|---------|-----------|--------------|---------------------|
| `get_timer_state` | None | `TimerState` | Once on mount |
| `skip_break` | None | `void` | When the skip button is pressed |

### 1.4 Tauri Event Subscriptions

| Event name | Payload type | Subscribe start | Unsubscribe |
|-----------|------------|---------|---------|
| `timer-tick` | `TimerState` | `onMount` | `onDestroy` |
| `break-end` | None | `onMount` | `onDestroy` |

### 1.5 Window Operations

| Operation | API | Condition |
|-----------|-----|-----------|
| Window close | `getCurrentWindow().close()` | When `break-end` event is received |

### 1.6 Type Dependencies

```typescript
type TimerPhase = "Focus" | "ShortBreak" | "LongBreak";

interface TimerState {
  phase: TimerPhase;
  paused: boolean;
  elapsed_secs: number;
  phase_duration_secs: number;
  short_break_count: number;
  settings: TimerSettings;
}
```

## 2. State

### 2.1 Reactive State (`$state`)

| Variable | Type | Initial value | Update trigger | Purpose |
|----------|------|---------------|---------------|---------|
| `remaining` | `string` | `"--:--"` | `timer-tick` event / initial fetch | Remaining time for UI display (MM:SS format) |
| `phase` | `TimerPhase` | `"ShortBreak"` | `timer-tick` event / initial fetch | Current phase (used for message selection) |
| `initialized` | `boolean` | `false` | When initial timer state fetch completes | Controls fade-in animation |

### 2.2 Non-reactive Variables

| Variable | Type | Purpose |
|----------|------|---------|
| `unlistenTick` | `(() => void) \| null` | Cleanup function for the `timer-tick` listener |
| `unlistenEnd` | `(() => void) \| null` | Cleanup function for the `break-end` listener |

### 2.3 Constants

| Name | Type | Content |
|------|------|---------|
| `messages` | `Record<TimerPhase, { title: string; subtitle: string }>` | Display message map per phase |

Message map details:

| Phase | title | subtitle |
|-------|-------|----------|
| `ShortBreak` | `"Rest your eyes"` (`break.short_title`) | `"Look into the distance and blink"` (`break.short_subtitle`) |
| `LongBreak` | `"Stand up and stretch"` (`break.long_title`) | `"Move your body and take a deep breath"` (`break.long_subtitle`) |
| `Focus` | `""` | `""` |

### 2.4 State Transition Diagram

```
[Mount]
  |
  +-- getTimerState() -> handleTick() -> initialized=true
  |
  +-- Register timer-tick event
  |
  +-- Register break-end event
        |
        +-- timer-tick --> Update remaining, phase (repeating)
        |
        +-- break-end --> window.close()

[Destroy]
  +-- unlistenTick(), unlistenEnd()
```

## 3. Logic

### 3.1 `handleTick(state: TimerState)`

1. Calculate remaining seconds via `remainingSecs(state)` (`phase_duration_secs - elapsed_secs`, clamped to 0)
2. Convert to `MM:SS` string with `formatTime()` and update `remaining`
3. Sync `phase` with `state.phase`
4. Set `initialized` to `true`

### 3.2 `handleSkip()`

- Calls `skipBreak()` (Tauri IPC `skip_break` command)
- Expects a phase transition on the Rust side, which will emit a `break-end` event

### 3.3 `handleBreakEnd()`

- Gets the current window reference via `getCurrentWindow()`
- Closes the overlay window with `win.close()`

### 3.4 Display Text Fallback

- If `messages[phase]?.title` is falsy -> display `"On break"` (`break.fallback_title`)
- If `messages[phase]?.subtitle` is falsy -> display empty string

## 4. Side Effects

### 4.1 On Mount (`onMount`)

| Order | Process | Side effect |
|-------|---------|-------------|
| 1 | `getTimerState()` | Tauri IPC call |
| 2 | `handleTick(state)` | State update (remaining, phase, initialized) |
| 3 | Register `onTimerTick(handleTick)` | Add Tauri event listener |
| 4 | Register `onBreakEnd(handleBreakEnd)` | Add Tauri event listener |

### 4.2 On Destroy (`onDestroy`)

| Process | Side effect |
|---------|-------------|
| `unlistenTick?.()` | Unsubscribe `timer-tick` listener |
| `unlistenEnd?.()` | Unsubscribe `break-end` listener |

### 4.3 User Actions

| Action | Side effect |
|--------|-------------|
| Skip button click | Issue `skip_break` IPC command |

### 4.4 UI Side Effects

| Trigger | Effect |
|---------|--------|
| `initialized` -> `true` | `.overlay.visible` class applied -> fadeIn animation starts (0.8s, ease-out) |
| `break-end` event | Window destroyed |

## 5. Notes

### 5.1 Design Intent

- **Initial display stability**: The `initialized` flag keeps content hidden until the initial timer state is fetched, preventing flickering of `"--:--"`. `getTimerState()` is called immediately in `onMount` to reflect state before event subscription begins.
- **Fade-in animation**: When `initialized` becomes `true`, a CSS animation (opacity 0->1, scale 0.96->1) triggers, causing the overlay to appear smoothly.
- **Empty messages for Focus phase**: The `messages` map has a Focus entry, but title/subtitle are empty strings. Since the overlay is only displayed during break phases, Focus values are not normally used. `"On break"` (`break.fallback_title`) is displayed as a fallback.

### 5.2 Assumptions & Constraints

- This component is displayed standalone within a Tauri overlay window. It operates in a separate window from the main window.
- Window lifecycle management (creation, fullscreen, presentation options) is handled by the Rust side (`lib.rs`).
- The `timer-tick` event emission frequency depends on the Rust-side timer (assumed to be 1-second intervals).
- The type cast `as unknown as () => void` is a workaround to resolve the type mismatch between the `listen()` return type and the unlisten variable type.

### 5.3 Testing Considerations

- Whether `remaining` and `phase` are correctly updated when a `timer-tick` event is received
- Whether the window is closed when a `break-end` event is received
- Whether listeners are properly unsubscribed on `onDestroy`
- Fallback display when a phase not in the `messages` map is received
- That the `.visible` class is not applied before initialization
