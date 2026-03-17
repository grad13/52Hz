---
updated: 2026-03-16 07:20
checked: -
Retired: -
Format: spec-v2.1
Source: tauri/src/tray.rs
---

# tray.rs spec

Runtime: Rust

## 0. Meta

| Item | Value |
|---|---|
| File | `code/app/tauri/src/tray.rs` |
| Responsibility | Building the macOS menu bar icon, click event handling, and title updates |
| Lines | ~265 (main ~204, tests ~61) |
| Visibility | `pub(crate)` -- crate-internal only |
| Dependencies | `tauri`, `objc2`, `objc2_app_kit`, `objc2_foundation`, `hover_poll` |
| Dependents | `lib.rs` (calls `build_tray` + `set_tray_icon_visible` during setup, calls `update_tray_title` every second), `commands.rs` (via `set_tray_icon_visible` command) |

## 1. Contract

### 1.1 `build_tray`

```rust
pub(crate) fn build_tray(
    app: &tauri::App,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>>
```

| Parameter | Type | Description |
|---|---|---|
| `app` | `&tauri::App` | Tauri app instance (only available during setup) |
| `title` | `&str` | Initial title string for the menu bar |

- **Return value:** `Result<(), Box<dyn Error>>` -- `()` on success
- **Invocation timing:** Called once at app startup
- **Side effects:** Stores `APP_HANDLE` and `STATUS_ITEM_PTR` globally

### 1.2 `set_tray_icon_visible`

```rust
pub(crate) fn set_tray_icon_visible(visible: bool)
```

- `visible: true` -> Re-set the icon (28.6x18.0pt, color, non-template)
- `visible: false` -> `button.setImage(None)` to remove the icon (title text only)
- Must be called from the main thread (via `run_on_main_thread`)
- No-op if `build_tray` has not been called

### 1.3 `update_tray_title`

```rust
pub(crate) fn update_tray_title(title: &str)
```

- Updates the menu bar title
- Must be called from the main thread (via `run_on_main_thread`)
- No-op if `build_tray` has not been called

## 2. State

### 2.1 Global State

| Variable | Type | Description |
|---|---|---|
| `APP_HANDLE` | `OnceLock<AppHandle>` | Tauri app handle (used for window operations from click handler) |
| `STATUS_ITEM_PTR` | `AtomicPtr<c_void>` | Raw pointer to `Retained<NSStatusItem>` (leaked for app lifetime) |
| _(moved)_ | `HOVER_POLLING` has been moved to `hover_poll.rs` |

### 2.2 NSStatusItem Attributes

| Attribute | Value |
|---|---|
| Icon | `icons/tray-icon.png` (28.6x18.0pt, color, non-template) |
| Title | Timer remaining time (updated every second) |
| Length | `NSVariableStatusItemLength` (auto width) |

### 2.3 External State Dependencies

- `"main"` window -- Used for popup show/hide on click

## 3. Logic

### 3.1 Click Handling (target-action via `TrayHandler`)

```
NSStatusBarButton click -> TrayHandler.handleClick:
  window = app.get_webview_window("main")
  if window.is_visible():
    window.hide()
  else:
    position_window_below_tray(window)
    window.show()
    window.set_focus()
    NSApplication.activateIgnoringOtherApps(true)
    hover_poll::start(app_handle)
```

### 3.1.1 Hover Polling (`hover_poll::start`)

Delegated to `hover_poll::start(app_handle)`. Logic is separated into the `hover_poll.rs` module.

Since Accessory apps do not receive native mouse events in WKWebView, the Rust side polls cursor position and injects synthetic `mouseenter`/`mouseleave` events into JS. See the `hover_poll.rs` spec for details.

- **Polling interval:** 50ms (20fps, negligible CPU load)
- **Stop condition:** Automatically stops when the window becomes hidden

### 3.2 Window Positioning (`position_window_below_tray`)

```
item.button -> button.window -> frame (Cocoa coordinates)
screen.frame.size.height for Cocoa -> Tauri coordinate conversion
window.set_position(Logical(x, btn_bottom_y))
```

- Cocoa coordinates: bottom-left origin -> Tauri coordinates: top-left origin conversion
- Positions window directly below the button, horizontally centered
- Popup window width is hardcoded to `320.0` (logical pixels) (must stay in sync with lib.rs window definition)

### 3.2.1 `calc_window_position` Formula

Pure function. Returns `(x, y)` from 5 arguments.

| Argument | Meaning |
|---|---|
| `btn_x` | Button window frame Cocoa x (left edge) |
| `btn_y` | Button window frame Cocoa y (bottom edge in bottom-left origin) |
| `btn_width` | Button window frame width |
| `screen_height` | Screen height (logical pixels) |
| `win_width` | Popup window width (currently fixed at 320.0) |

| Output | Formula | Meaning |
|---|---|---|
| `y` | `screen_height - btn_y` | Cocoa bottom-left origin -> top-left origin conversion |
| `x` | `btn_x + btn_width / 2.0 - win_width / 2.0` | Center popup on button center |

### 3.3 `TrayHandler` (ObjC Class)

- `NSObject` subclass defined via `define_class!` (name: `Hz52TrayHandler`)
- `handleClick:` selector calls `handle_tray_click()`
- Leaked for app lifetime with `std::mem::forget` (target is a weak reference)

## 4. Side Effects

### 4.1 Window Operations

| Operation | Location | Description |
|---|---|---|
| `window.show()` / `window.hide()` | `handle_tray_click` | Toggle main window visibility |
| `window.set_position()` | `position_window_below_tray` | Position window directly below menu bar icon |
| `window.set_focus()` | `handle_tray_click` | Move focus to window |

### 4.2 macOS Native API

| Operation | Description |
|---|---|
| `NSStatusBar::systemStatusBar()` | Get system status bar |
| `statusItemWithLength(-1.0)` | Add status item to menu bar |
| `NSImage::initWithData` + `setSize(28.6, 18.0)` | Set icon image at 28.6x18.0pt (height 18pt is menu bar standard, width 28.6pt matches whale shape aspect ratio) |
| `button.setTarget` / `setAction` | Connect click handler |
| `NSApplication::activateIgnoringOtherApps` | Activate app (acquire focus) |
| `NSEvent::mouseLocation` | Get cursor position (for hover polling, no events needed) |
| `window.eval()` | Inject synthetic JS events (hover polling) |

### 4.3 unsafe Code

| Location | Reason |
|---|---|
| `MainThreadMarker::new_unchecked()` | Click handler and title update run on the main thread |
| `button.setTarget` / `button.setAction` | ObjC target-action pattern (selector existence and target validity guaranteed) |
| `Retained::into_raw` / raw pointer reference | Global retention of NSStatusItem (main thread access only) |

## 5. Tests

### 5.1 Unit Tests (`#[cfg(test)]` inline, 6 tests)

| Test | Verification |
|---|---|
| `window_centered_below_button` | Basic coordinate conversion and horizontal centering |
| `window_position_standard_menu_bar` | Positioning with typical menu bar height (24pt) |
| `window_can_extend_left_of_screen` | When button is at left edge, allows x to be negative |
| `window_position_retina_screen` | Coordinate conversion at Retina resolution |
| `narrow_button_still_centers_window` | Centering with icon-only button (width 24pt) |
| `wide_button_with_long_title` | Centering with titled button (width 200pt) |

### 5.2 Test Strategy

- `calc_window_position`: Extracted as a pure function, unit testable
- Native API calls (NSStatusBar, NSImage, setTarget, etc.): Not under test, manual verification

## 6. Notes

### 6.1 Design Choices

- **Not using Tauri TrayIcon:** The `tray-icon` crate hardcodes icon height to 18pt, so NSStatusItem is created directly to achieve the 28.6x18.0pt icon
- **Not using tauri-plugin-positioner:** Since NSStatusItem does not produce Tauri tray events, window position is calculated manually from the button's screen frame
- **Global raw pointer:** `Retained<NSStatusItem>` is `!Send` so it cannot go in `OnceLock`. Safety is ensured via `AtomicPtr` + main-thread-only access

### 6.2 Error Handling

- `build_tray`: Called at startup, so failures propagate via `Result` to the caller (fatal error)
- `update_tray_title`: Returns silently when global is uninitialized (no-op)

### 6.3 Window ID

- Popup window: `"main"` -- The webview window created in `lib.rs`
