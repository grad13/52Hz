---
updated: 2026-03-16 07:20
checked: -
Retired: -
Format: spec-v2.1
Source: tauri/src/overlay.rs
---

# overlay.rs spec

## 0. Meta

| Item | Value |
|---|---|
| Module | `overlay` |
| File | `code/app/tauri/src/overlay.rs` |
| Runtime | Rust (Tauri v2) |
| Lines | 102 |
| Responsibility | Creation/destruction of the break overlay window and macOS UI lock control |

## 1. Contract

### 1.1 Public Functions

#### `create_break_overlay`

```rust
pub(crate) fn create_break_overlay(
    app: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>>
```

| Item | Description |
|---|---|
| Visibility | `pub(crate)` |
| Parameter | `app: &tauri::AppHandle` -- Tauri application handle |
| Return value | `Result<(), Box<dyn std::error::Error>>` |
| Precondition | Must be called from the main thread (assumed by the macOS unsafe blocks) |
| Postcondition | `"break-overlay"` window covers the entire screen and UI lock is active |
| Side effects | Window creation, NSWindow level setting, NSApplicationPresentationOptions change |

**Error conditions**:
- `WebviewWindowBuilder::build()` failure
- `overlay.current_monitor()` failure
- `overlay.ns_window()` failure

**Headless mode**: When the `FIFTYTWOHZ_HEADLESS` environment variable is set, skips window creation and only outputs debug log, returning `Ok(())`.

#### `unlock_presentation`

```rust
pub(crate) fn unlock_presentation()
```

| Item | Description |
|---|---|
| Visibility | `pub(crate)` |
| Parameters | None |
| Return value | None |
| Precondition | Must be called from the main thread |
| Postcondition | macOS presentation options revert to `Default` |
| Side effects | `NSApplicationPresentationOptions` change |

### 1.2 Window ID

| Constant | Value | Purpose |
|---|---|---|
| (hardcoded) | `"break-overlay"` | Unique identifier for the overlay window |

### 1.3 Environment Variables

| Name | Type | Default | Purpose |
|---|---|---|---|
| `FIFTYTWOHZ_HEADLESS` | Existence check (`is_ok()`) | Not set | Skips window creation when set (for testing) |

## 2. State

This module itself holds no internal state. State is externalized to the OS-side window and application settings.

### 2.1 Implicit State: Overlay Visible/Hidden

```
[Hidden] --create_break_overlay()--> [Visible + UI Locked]
[Visible + UI Locked] --window close + unlock_presentation()--> [Hidden]
```

| State | Overlay Window | PresentationOptions |
|---|---|---|
| Hidden | Does not exist or closed | `Default` |
| Visible + UI Locked | `"break-overlay"` exists, Level=1000, displayed on all Spaces | `HideDock \| HideMenuBar \| DisableProcessSwitching \| DisableHideApplication \| DisableForceQuit` |

### 2.2 Caller Responsibilities

- `create_break_overlay` and `unlock_presentation` must be used as a pair
- Window close is managed by the caller (`lib.rs`'s `end_break` command, etc.)
- `unlock_presentation` is independent of window close and must be called separately

## 3. Logic

### 3.1 create_break_overlay Processing Flow

```
START
  |
  v
Is FIFTYTWOHZ_HEADLESS set?
  |-- Yes --> Output debug log --> return Ok(())
  |-- No
  v
Does existing "break-overlay" window exist?
  |-- Yes --> close()
  |-- No
  v
Build new window with WebviewWindowBuilder
  - URL: index.html?view=break
  - decorations: false
  - skip_taskbar: true
  - closable/minimizable/maximizable/resizable: false
  - focused: true
  - background_color: #0a0a0e
  |
  v
[macOS] Get monitor information
  - Set window position to monitor origin
  - Set window size to monitor size
  |
  v
[macOS] NSWindow configuration
  - setLevel(1000)                    ... Topmost
  - setCollectionBehavior              ... CanJoinAllSpaces | Stationary
  |
  v
[macOS] NSApplication PresentationOptions configuration
  - HideDock
  - HideMenuBar
  - DisableProcessSwitching
  - DisableHideApplication
  - DisableForceQuit
  |
  v
[non-macOS] set_fullscreen(true)
  |
  v
return Ok(())
```

### 3.2 unlock_presentation Processing Flow

```
START
  |
  v
[macOS] Set NSApplication PresentationOptions to Default
  |
  v
Output debug log
  |
  v
END
```

### 3.3 Window Attribute List

| Attribute | Value | Reason |
|---|---|---|
| title | `""` | No title bar needed |
| decorations | `false` | Borderless window |
| skip_taskbar | `true` | Do not show in taskbar |
| closable | `false` | Prevent user from closing |
| minimizable | `false` | Minimize not allowed |
| maximizable | `false` | Maximize button not needed |
| resizable | `false` | Resize not allowed |
| focused | `true` | Acquire focus on creation |
| background_color | `(10, 10, 14, 255)` = `#0a0a0e` | Prevent white flash |

### 3.4 NSWindow Level

`setLevel(1000)` is equivalent to `kCGMaximumWindowLevel (CGWindowLevelKey)`. This is at or above the level of screensavers and system alerts, displayed above all other application windows.

## 4. Side Effects

### 4.1 NSWindow Operations (macOS)

| Operation | API | Effect |
|---|---|---|
| Window level setting | `NSWindow::setLevel(1000)` | Places window at the topmost level above all windows |
| Collection behavior setting | `NSWindow::setCollectionBehavior(CanJoinAllSpaces \| Stationary)` | Pins display across all virtual desktops (Spaces) |

### 4.2 NSApplication Operations (macOS)

| Operation | API | Effect |
|---|---|---|
| UI lock | `NSApplication::setPresentationOptions(HideDock \| HideMenuBar \| DisableProcessSwitching \| DisableHideApplication \| DisableForceQuit)` | Hides Dock, hides menu bar, disables Cmd+Tab, disables Cmd+H, disables Force Quit |
| UI unlock | `NSApplication::setPresentationOptions(Default)` | Removes all UI restrictions |

### 4.3 Tauri Window Operations

| Operation | Condition | Effect |
|---|---|---|
| Existing window close | If `"break-overlay"` exists | Destroy old overlay |
| New window creation | Always (except headless) | Create `"break-overlay"` WebviewWindow |
| Monitor position/size setting | macOS and successful monitor retrieval | Expand window to cover entire monitor |
| Fullscreen | non-macOS | OS standard fullscreen mode |

### 4.4 Log Output (stderr)

| Message | Condition |
|---|---|
| `[52Hz] presentation-options → locked` | On `create_break_overlay` success (`debug_assertions` only) |
| `[52Hz] presentation-options → default` | On `unlock_presentation` invocation (`debug_assertions` only) |

### 4.5 Justification for unsafe

| Location | unsafe Operation | Safety Justification |
|---|---|---|
| L58 | `MainThreadMarker::new_unchecked()` | Assumes invocation from `run_on_main_thread` (documented in comments) |
| L60-61 | `Retained::retain(ns_window as *mut NSWindow)` | Called on a valid pointer returned by `ns_window()` |
| L95 (`unlock_presentation`) | `MainThreadMarker::new_unchecked()` | Same as above, assumes invocation from main thread |

## 5. Notes

### 5.1 Design Decisions

- **Window level 1000**: Uses a value close to macOS's `kCGMaximumWindowLevel` to ensure display above any other application. As a DeskRest clone, this design physically prevents the user from continuing work during breaks.
- **PresentationOptions lock**: Disables Dock, menu bar, process switching, app hiding, and Force Quit to block all "escape routes" from the break.
- **White flash prevention**: Sets `background_color` to a dark color (`#0a0a0e`) to prevent a white screen from briefly appearing before the Webview rendering completes.
- **Headless mode**: A mechanism for verifying logic flow in integration tests without creating actual windows. Controlled by the `FIFTYTWOHZ_HEADLESS` environment variable.

### 5.2 Platform Support

- **macOS**: Directly manipulates NSWindow / NSApplication Objective-C APIs via the `objc2` crate. Full control over all-Spaces display, window level, and presentation options.
- **non-macOS**: Only `set_fullscreen(true)` from Tauri. UI lock (PresentationOptions equivalent) is not implemented.

### 5.3 Relationship with Callers

- `create_break_overlay` is called from the break-start processing in `lib.rs`
- `unlock_presentation` is called from the break-end processing in `lib.rs`
- Window close (destruction of `"break-overlay"`) is handled separately by the caller

### 5.4 Known Limitations

- In multi-monitor environments, only the primary monitor (returned by `current_monitor()`) is covered
- On non-macOS environments, there is no UI lock mechanism, so breaks can be bypassed via Alt+Tab etc.
- `setLevel(1000)` may be occluded if another app uses an equal or higher level
