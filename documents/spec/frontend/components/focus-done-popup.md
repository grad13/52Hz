---
updated: 2026-03-15 09:02
checked: -
Deprecated: 2026-03-14
Format: spec-v2.1
Source: (deleted) frontend/components/FocusDonePopup.svelte
---

# FocusDonePopup spec

## 0. Meta

| key | value |
|-----|-------|
| Module | `frontend/components/FocusDonePopup.svelte` |
| Runtime | JS-ESM (Svelte 5 component) |
| Responsibility | A popup for choices after Focus completion. Presents three action types to the user: take a break / skip / extend, then issues the corresponding IPC command and closes the window upon selection. |
| Dependencies | `@tauri-apps/api/window` (`getCurrentWindow`), `../lib/timer` (`acceptBreak`, `extendFocus`, `skipBreakFromFocus`) |

## 1. Contract

### 1.1 Props

None. This component is the root component of the focus-done-popup window and does not receive props.

### 1.2 Public Events

None. No custom events are emitted.

### 1.3 Tauri IPC Command Calls

| Command (wrapper function) | Arguments | Return value | Invocation condition |
|---------------------------|-----------|--------------|---------------------|
| `acceptBreak()` | None | `void` | When the "Take a break" (`focus_done.accept`) button is pressed |
| `skipBreakFromFocus()` | None | `void` | When the "Skip" (`focus_done.skip`) button is pressed |
| `extendFocus(secs)` | `secs: number` | `void` | When "+1 min" / "+3 min" / "+5 min" button is pressed |

### 1.4 Tauri Event Subscriptions

None. This component does not subscribe to Tauri events.

### 1.5 Window Operations

| Operation | API | Condition |
|-----------|-----|-----------|
| Window close | `getCurrentWindow().close()` | Executed after each action handler completes the IPC call |

Each handler `await`s the IPC command before calling `getCurrentWindow().close()` (guaranteeing IPC -> close ordering).

### 1.6 Type Dependencies

None. This component does not directly reference any type definitions.

## 2. State

### 2.1 Reactive State (`$state`)

None. This component has no reactive state variables.

### 2.2 Non-reactive Variables

None.

### 2.3 Constants

None.

## 3. Logic

### 3.1 `handleAcceptBreak()`

1. Calls `acceptBreak()` with await (Tauri IPC)
2. Closes the window with `getCurrentWindow().close()`

### 3.2 `handleExtend(secs: number)`

1. Calls `extendFocus(secs)` with await (Tauri IPC)
2. Closes the window with `getCurrentWindow().close()`

`secs` values per button:

| Button | secs |
|--------|------|
| +1 min | 60 |
| +3 min | 180 |
| +5 min | 300 |

### 3.3 `handleSkip()`

1. Calls `skipBreakFromFocus()` with await (Tauri IPC)
2. Closes the window with `getCurrentWindow().close()`

## 4. UI Structure

### 4.1 Layout

```
.popup (flex-column, center, height: 100%, gap: 0.6rem)
+-- h3 "Focus Complete"
+-- p.message "Great work! What would you like to do next?" (`focus_done.message`)
+-- .actions (flex-row, gap: 0.5rem, width: 100%)
|   +-- button.btn.primary "Take a break" (`focus_done.accept`) -> handleAcceptBreak()
|   +-- button.btn "Skip" (`focus_done.skip`) -> handleSkip()
+-- .extend-actions (flex-row, gap: 0.4rem)
    +-- button.btn-extend "+1 min" -> handleExtend(60)
    +-- button.btn-extend "+3 min" -> handleExtend(180)
    +-- button.btn-extend "+5 min" -> handleExtend(300)
```

### 4.2 CSS Variables

| CSS variable | Usage |
|-------------|-------|
| `--text` | Text color for `h3`, text color for `.btn` |
| `--text-secondary` | Text color for `.message`, text color for `.btn-extend` |
| `--bg-secondary` | Background color for `.btn` |
| `--success` | Background color for `.btn.primary` |

### 4.3 Button Style Classification

| Class | Purpose | Characteristics |
|-------|---------|-----------------|
| `.btn.primary` | "Take a break" (`focus_done.accept`) | `--success` background, text color `#1a1a2e`, font-weight: 600, no border |
| `.btn` | "Skip" (`focus_done.skip`) | `--bg-secondary` background, `--text` text color, border: 1px solid rgba(255,255,255,0.15) |
| `.btn-extend` | "+1 min" / "+3 min" / "+5 min" | transparent background, `--text-secondary` text color, border: 1px solid rgba(255,255,255,0.1) |

## 5. Side Effects

### 5.1 On Mount (`onMount`)

None. This component has no side effects on mount.

### 5.2 On Destroy (`onDestroy`)

None. No listener registration, so no cleanup processing is needed.

### 5.3 User Actions

| Action | Side effect |
|--------|-------------|
| "Take a break" (`focus_done.accept`) button click | Issue `accept_break` IPC command -> window close |
| "Skip" (`focus_done.skip`) button click | Issue `skip_break_from_focus` IPC command -> window close |
| "+1 min" button click | Issue `extend_focus(60)` IPC command -> window close |
| "+3 min" button click | Issue `extend_focus(180)` IPC command -> window close |
| "+5 min" button click | Issue `extend_focus(300)` IPC command -> window close |

## 6. Notes

### 6.1 Design Intent

- **Simple stateless design**: This component has no reactive state at all. It is specialized in the responsibility of receiving the user's selection, issuing an IPC command, and closing the window.
- **IPC -> close ordering guarantee**: Each handler uses `async/await` to wait for the IPC command to complete before closing the window. This guarantees the window disappears only after the state change on the Rust side is confirmed.
- **Generic handler for extend buttons**: `handleExtend(secs: number)` is defined as a shared handler, and each button calls it via an inline function `() => handleExtend(N)`. This structure makes it easy to add additional durations.

### 6.2 Assumptions & Constraints

- This component is displayed standalone within a Tauri focus-done-popup window. It operates in a separate window from the main window.
- Window lifecycle management (creation, display) is handled by the Rust side (`lib.rs`).
- The popup window is expected to be created by the Rust side when the Focus phase completes; this component itself does not control when the window is created.

### 6.3 Deprecation Notice

This component (`FocusDonePopup.svelte`) has been deleted. The focus-done UI has been migrated to a toast card within `Toast.svelte`. The Rust side now emits `"focus-done-toast"` (via `event_handlers.rs`) instead of creating a `WebviewWindow`. The toast card is rendered inline in the `Toast.svelte` stack.

### 6.4 Testing Considerations

- Whether `acceptBreak()` is called when the "Take a break" button is pressed, and the window is subsequently closed
- Whether `skipBreakFromFocus()` is called when the "Skip" button is pressed, and the window is subsequently closed
- Whether `extendFocus()` is called with the correct seconds (60/180/300) for each extend button press
- Whether the window is not closed before the IPC call completes (verify await ordering)
