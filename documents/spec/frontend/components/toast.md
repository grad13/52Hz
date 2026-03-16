---
Created: 2026-03-06
Updated: 2026-03-08
Verified: -
Deprecated: -
Format: spec-v2.1
Source: frontend/components/Toast.svelte
---

# Toast Component Spec

## 1. Overview

Root component of a transparent overlay window that manages a stack of presence notification cards and focus-done action cards.

- Runtime: JS-ESM (Svelte 5 component)
- File: `code/app/frontend/components/Toast.svelte`
- Props: None (root component of the toast window)

## 2. Responsibility

- Receive presence notifications and focus-done notifications via Tauri events and display them as a card stack
- Toggle toast display enabled/disabled
- Control animation direction and stack ordering based on display position (4 corners)
- Sync window size and delegate positioning to the Rust side

### Out of Scope

- Actual on-screen window positioning (Rust side processes it upon receiving the `presence-reposition` event)
- Presence message generation
- Settings persistence

## 3. Type Definitions

### 3.1 ToastMessage

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string` | Sender name |
| `message` | `string` | Message body |

### 3.2 ToastItem

| Field | Type | Description |
|-------|------|-------------|
| `id` | `number` | Unique identifier |
| `type` | `"toast"` | Literal type |
| `msg` | `ToastMessage` | Display message |
| `leaving` | `boolean` | Exit animation in progress flag |

### 3.3 FocusDoneItem

| Field | Type | Description |
|-------|------|-------------|
| `id` | `number` | Unique identifier |
| `type` | `"focus-done"` | Literal type |
| `leaving` | `boolean` | Exit animation in progress flag |

### 3.4 StackItem

Union type of `ToastItem | FocusDoneItem`. Discriminated by the `type` field.

## 4. Constants

| Constant name | Value | Description |
|---------------|-------|-------------|
| `DISPLAY_MS` | `180000` (3 min) | Time before automatic toast dismissal |
| `TOAST_H` | `58` | Normal toast card height (px) |
| `FOCUS_DONE_H` | `100` | Focus-done card height (px) |
| `GAP` | `6` | Gap between cards (px) |
| `PAD` | `8` | Top/bottom padding of the stack (px) |
| `WIN_W` | `276` | Window width (px) |

## 5. State

| Variable name | Type | Reactive | Description |
|---------------|------|----------|-------------|
| `items` | `StackItem[]` | `$state` | Currently displayed item stack |
| `enabled` | `boolean` | `$state` | Toast display enabled/disabled |
| `position` | `PresencePosition` | `$state` | Display position (`top-right`, `top-left`, `bottom-right`, `bottom-left`) |
| `nextId` | `number` | - | Next item ID to assign |
| `level` | `PresenceLevel` | `$state` | Display level (`"always-front"`, `"dynamic"`, `"always-back"`) |
| `maxToasts` | `number` | `$state` | Maximum number of simultaneously active items (default 4, configurable via settings) |
| `showIcon` | `boolean` | `$state` | Show/hide whale SVG background icon (default `true`) |
| `likeIcon` | `PresenceLikeIcon` | `$state` | Like icon type (`"heart"` \| `"star"` \| `"none"`, default `"heart"`) |
| `raised` | `boolean` | - | Whether temporarily raised to front in `"always-back"` mode |
| `shown` | `boolean` | - | Whether the window is currently shown (prevents Z-order changes from redundant `show()` calls) |
| `needsRaise` | `boolean` | - | Whether to raise on the first click in `"always-back"` mode |
| `timers` | `Map<number, ReturnType<typeof setTimeout>>` | - | Map of item ID -> auto-dismiss timer |

## 6. Events

### 6.1 Received Events (listen)

| Event name | Payload type | Handler | Description |
|-----------|-------------|---------|-------------|
| `presence-message` | `ToastMessage` | `addToast` | Add a new presence toast |
| `presence-toast-toggle` | `boolean` | - | Update `enabled`. When `false`, `dismissAll` all toasts |
| `presence-toast-click` | None | - | `dismiss` the oldest active `toast`-type item by chronological order |
| `focus-done-toast` | None | `addFocusDone` | Add a focus-done card |
| `presence-position-change` | `string` (as `PresencePosition`) | - | Update `position` |
| `presence-level-setting` | `string` (as `PresenceLevel`) | - | Update `level`, reset `raised`, set `needsRaise` back to `true` |
| `presence-max-toasts-change` | `number` | - | Update `maxToasts` |
| `presence-show-icon-change` | `boolean` | - | Update `showIcon` |
| `presence-like-icon-change` | `string` (as `PresenceLikeIcon`) | - | Update `likeIcon`, reset `hasLikedThisSession` and `likedId` |

### 6.2 Emitted Events (emit)

| Event name | Payload type | Trigger | Description |
|-----------|-------------|---------|-------------|
| `presence-reposition` | `PresencePosition` | When `syncWindow` executes | Notify the Rust side of the current position and request window position recalculation |
| `presence-level-change` | `string` | `raise()` / `restoreIfNeeded()` / `syncWindow()` (after show) / `presence-toast-click` (after raise) | Request NSWindow level change from the Rust side |

## 7. Logic

### 7.1 addToast

1. Do nothing if `enabled` is `false`
2. If the active item count is >= `maxToasts`, `dismiss` the oldest `toast`-type item (the array is always in chronological order, so `find` from the beginning locates the oldest)
3. Generate a new `ToastItem`
4. Append to the end of the array (always append to end regardless of position)
5. Call `syncWindow`
6. Set a timer for automatic `dismiss` after `DISPLAY_MS`

### 7.2 addFocusDone

1. If an existing active `focus-done` item exists, `dismiss` it first
2. Generate a new `FocusDoneItem`
3. Append to the end of the array (always append to end regardless of position)
4. Call `syncWindow`
5. No auto-dismiss timer is set (remains until the user selects an action)

### 7.3 dismiss

1. Clear the timer for the item
2. Set `leaving: true` (start exit animation)
3. After 350ms, remove the item from the array and call `syncWindow`

### 7.4 dismissAll

- Clear timers for all `toast`-type items and remove them from the array
- `focus-done`-type items remain
- Call `syncWindow`

### 7.5 syncWindow

1. Filter for active items (`leaving: false`)
2. If 0 active items, hide the window (reset the shown flag)
3. If 1 or more, calculate total height and set window size with `LogicalSize`
4. Emit `presence-reposition` event (with position as payload)
5. If the window is not yet shown, `show()` it, set the shown flag, and emit `presence-level-change` to re-apply the NSWindow level (since `show()` resets the level). If already shown, only resize (no Z-order change)

### 7.6 raise (Level Elevation)

Processing to draw the user's attention when a focus-done card is displayed.

| Mode | Behavior |
|------|----------|
| `"always-back"` | Set `raised=true` and emit `emit("presence-level-change", "always-front")` to elevate to level 25 |
| `"dynamic"` | Call `win.show()` to bring to front (level stays at 0) |
| `"always-front"` | Do nothing (already in front) |

### 7.7 restoreIfNeeded (Level Restoration)

Restores the original level after a focus-done card is dismissed. Only executes when `raised` is `true`.

- Set `raised=false` and emit `emit("presence-level-change", "always-back")` to restore level to -1

### 7.8 presence-toast-click Handler

| Mode | needsRaise | Behavior |
|------|-----------|----------|
| `"always-back"` | `true` | `raise()` + `await win.show()` + `await emit("presence-level-change", "always-front")` + `needsRaise=false` (first click raises to front; re-applies level after `show()` to ensure foregrounding) |
| `"always-back"` | `false` | `dismiss` the oldest toast |
| Other | - | `dismiss` the oldest toast |

### 7.9 Focus-Done Actions

| Button | Label | Processing |
|--------|-------|------------|
| primary | "Take a break" (`focus_done.accept`) | `acceptBreak()` -> `dismiss(id)` |
| secondary | "Skip" (`focus_done.skip`) | `skipBreakFromFocus()` -> `dismiss(id)` |

## 8. Position-based Behavior

The array is always in chronological order (head = oldest, tail = newest). New items are always appended to the array tail.
For `bottom-*`, CSS `flex-direction: column-reverse` reverses the display order, anchoring to the bottom edge of the screen.

| position | CSS flex-direction | New item insert position | Visual growth direction | Slide animation direction |
|----------|-------------------|------------------------|------------------------|--------------------------|
| `top-right` | `column` | Array tail | Downward | From right |
| `top-left` | `column` | Array tail | Downward | From left |
| `bottom-right` | `column-reverse` | Array tail | Upward | From right |
| `bottom-left` | `column-reverse` | Array tail | Upward | From left |

## 9. UI Structure

### 9.1 Toast Card (`toast` type)

- `<div>` element (`role="button"`), click to `dismiss`
- Whale SVG background: shown only when `showIcon` is `true` and not liked
- Liked background (`bg-like`): shown when `likedId` matches and `likeIcon !== "none"`. If `likeIcon` is `"star"`, displays a star (#f0c040); if `"heart"`, displays a heart (#e8547a)
- Like button: shown only when `likeIcon !== "none"` and not yet liked. Icon matches `likeIcon` (heart or star)
- `.name`: Sender name (0.68rem, 600 weight)
- `.msg`: Message body (0.78rem)

### 9.2 Focus-Done Card (`focus-done` type)

- `<div>` element, no click-to-dismiss
- `.label`: `"Session complete"` (`toast.session_complete`) (0.68rem, 600 weight)
- `.msg`: `"Great work! What would you like to do next?"` (`focus_done.message`) (0.78rem)
- `.actions`: 2 buttons ("Take a break" (`focus_done.accept`) primary, "Skip" (`focus_done.skip`) secondary)

## 10. Lifecycle

### 10.1 onMount

1. Load initial value of `enabled` with `loadPresenceToast()`
2. Load initial value of `position` with `loadPresencePosition()`
3. Load initial value of `level` with `loadPresenceLevel()`
4. Load initial value of `maxToasts` with `loadPresenceMaxToasts()`
5. Load initial value of `showIcon` with `loadPresenceShowIcon()`
6. Load initial value of `likeIcon` with `loadPresenceLikeIcon()`
7. Register 9 event listeners

### 10.2 onDestroy

1. Unregister 9 event listeners
2. Clear all timers

## 11. Dependencies

### 11.1 External Dependencies

| Module | Purpose |
|--------|---------|
| `@tauri-apps/api/event` | `listen`, `emit` |
| `@tauri-apps/api/window` | `getCurrentWindow` |
| `@tauri-apps/api/dpi` | `LogicalSize` |

### 11.2 Internal Dependencies

| Module | Imports | Purpose |
|--------|---------|---------|
| `../lib/settings-store` | `loadPresenceToast`, `loadPresencePosition`, `loadPresenceLevel`, `loadPresenceMaxToasts`, `loadPresenceShowIcon`, `loadPresenceLikeIcon`, `PresencePosition`, `PresenceLevel`, `PresenceLikeIcon` | Load initial settings |
| `../lib/timer` | `acceptBreak`, `skipBreakFromFocus` | Execute focus-done actions |

## 12. Style Spec (Not Subject to Testing)

### 12.1 CSS Variable Dependencies

| Variable name | Purpose |
|---------------|---------|
| `--bg-secondary` | Card background color |
| `--border` | Card border color |
| `--text-secondary` | name / label text color |
| `--text` | msg text color |
| `--success` | Primary button background color |
| `--border-hover` | Secondary button hover border color |

### 12.2 Animations

| Name | Purpose | Direction |
|------|---------|-----------|
| `slide-in-right` | Card appearance for right-side position | `translateX(100%)` -> `translateX(0)` |
| `slide-out-right` | Card exit for right-side position | `translateX(0)` -> `translateX(100%)` |
| `slide-in-left` | Card appearance for left-side position | `translateX(-100%)` -> `translateX(0)` |
| `slide-out-left` | Card exit for left-side position | `translateX(0)` -> `translateX(-100%)` |

Animation timing: appearance 0.3s (`cubic-bezier(0.16, 1, 0.3, 1)`), exit 0.3s (`cubic-bezier(0.7, 0, 0.84, 0)`).

## 13. Constraints & Assumptions

- Uses Svelte 5 runes API (`$state`)
- Scoped CSS (Svelte `<style>` tag) keeps styles contained within the component
- Assumes operation within a Tauri WebView (browser compatibility is not considered)
- The toast window is expected to be created as a transparent background window on the Rust side
- Window position calculation and application is the Rust side's responsibility (via `presence-reposition` event)
