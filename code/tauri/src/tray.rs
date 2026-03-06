// meta: checked=2026-03-07
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::OnceLock;

use tauri::Manager;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, sel, AnyThread, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSImage, NSStatusBar, NSStatusItem};
use objc2_foundation::{NSData, NSObject, NSObjectProtocol, NSSize, NSString};

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
static STATUS_ITEM_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static HOVER_POLLING: AtomicBool = AtomicBool::new(false);

fn get_status_item<'a>() -> Option<&'a NSStatusItem> {
    let ptr = STATUS_ITEM_PTR.load(Ordering::Acquire);
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &*(ptr as *const NSStatusItem) })
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "Hz52TrayHandler"]
    struct TrayHandler;

    unsafe impl NSObjectProtocol for TrayHandler {}

    impl TrayHandler {
        #[unsafe(method(handleClick:))]
        fn handle_click(&self, _sender: &AnyObject) {
            handle_tray_click();
        }
    }
);

impl TrayHandler {
    fn new() -> Retained<Self> {
        let this = Self::alloc();
        unsafe { msg_send![this, init] }
    }
}

/// Poll cursor position every 50ms and inject synthetic hover events
/// into the WKWebView. Stops when the window becomes hidden.
fn start_hover_poll(app_handle: tauri::AppHandle) {
    // Prevent multiple polling threads
    if HOVER_POLLING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        let mut was_over = false;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let Some(window) = app_handle.get_webview_window("main") else {
                break;
            };
            if !window.is_visible().unwrap_or(false) {
                break;
            }
            // NSEvent.mouseLocation — always available, no events needed
            let mouse_screen: [f64; 2] =
                unsafe { msg_send![objc2::class!(NSEvent), mouseLocation] };

            // Get window frame to convert screen → window coords
            let win_frame: [[f64; 2]; 2] = match window.ns_window() {
                Ok(ns_window) => unsafe {
                    let ns_win = &*(ns_window as *const objc2_app_kit::NSWindow);
                    let f = ns_win.frame();
                    [
                        [f.origin.x, f.origin.y],
                        [f.size.width, f.size.height],
                    ]
                },
                Err(_) => break,
            };

            let origin_x = win_frame[0][0];
            let origin_y = win_frame[0][1];
            let width = win_frame[1][0];
            let height = win_frame[1][1];

            // Screen → window coordinates (Cocoa bottom-left → web top-left)
            let x = mouse_screen[0] - origin_x;
            let y = height - (mouse_screen[1] - origin_y);

            let in_window = x >= 0.0 && x <= width && y >= 0.0 && y <= height;
            if !in_window {
                if was_over {
                    let _ = window.eval(
                        "(() => { \
                            const el = document.querySelector('.timer-hover'); \
                            if (el) el.dispatchEvent(new MouseEvent('mouseleave', {bubbles:false})); \
                        })()"
                    );
                }
                was_over = false;
                continue;
            }

            // Hit-test against the actual .timer-hover element bounds
            let _ = window.eval(&format!(
                "(() => {{ \
                    const el = document.querySelector('.timer-hover'); \
                    if (!el) return; \
                    const r = el.getBoundingClientRect(); \
                    const over = {x} >= r.left && {x} <= r.right && {y} >= r.top && {y} <= r.bottom; \
                    if (over && !el.dataset.hp) {{ \
                        el.dataset.hp = '1'; \
                        el.dispatchEvent(new MouseEvent('mouseenter', {{bubbles:false}})); \
                    }} else if (!over && el.dataset.hp) {{ \
                        delete el.dataset.hp; \
                        el.dispatchEvent(new MouseEvent('mouseleave', {{bubbles:false}})); \
                    }} \
                }})()",
                x = x,
                y = y
            ));
            was_over = true;
        }
        HOVER_POLLING.store(false, Ordering::SeqCst);
    });
}

fn handle_tray_click() {
    let Some(app_handle) = APP_HANDLE.get() else {
        return;
    };
    let Some(window) = app_handle.get_webview_window("main") else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        position_window_below_tray(&window);
        let _ = window.show();
        let _ = window.set_focus();

        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let ns_app = NSApplication::sharedApplication(mtm);
        #[allow(deprecated)]
        ns_app.activateIgnoringOtherApps(true);

        // Accessory apps don't receive native mouse-move events
        // in WKWebView. Workaround: poll cursor position from Cocoa
        // and inject synthetic mouseenter/mouseleave via JS.
        start_hover_poll(app_handle.clone());
    }
}

/// Calculate window position centered below a menu bar button.
/// Cocoa uses bottom-left origin; this converts to top-left origin.
///
/// Returns `(x, y)` in top-left screen coordinates.
fn calc_window_position(
    btn_x: f64,
    btn_y: f64,
    btn_width: f64,
    screen_height: f64,
    win_width: f64,
) -> (f64, f64) {
    // In Cocoa coords, btn_y is the bottom-left of the button's window frame.
    // The bottom of the button in top-left coords = screen_height - btn_y.
    let y = screen_height - btn_y;
    // Center the popup horizontally below the button
    let x = btn_x + btn_width / 2.0 - win_width / 2.0;
    (x, y)
}

fn position_window_below_tray(window: &tauri::WebviewWindow) {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let Some(item) = get_status_item() else {
        return;
    };
    let Some(button) = item.button(mtm) else {
        return;
    };

    let Some(btn_window) = button.window() else {
        return;
    };
    let frame = btn_window.frame();

    if let Some(screen) = btn_window.screen() {
        let screen_h = screen.frame().size.height;
        let (x, y) = calc_window_position(
            frame.origin.x,
            frame.origin.y,
            frame.size.width,
            screen_h,
            320.0,
        );
        let _ = window.set_position(tauri::Position::Logical(
            tauri::LogicalPosition::new(x, y),
        ));
    }
}

pub(crate) fn build_tray(
    app: &tauri::App,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    APP_HANDLE.set(app.handle().clone()).ok();

    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let status_bar = NSStatusBar::systemStatusBar();

    // NSVariableStatusItemLength = -1.0
    let item = status_bar.statusItemWithLength(-1.0);

    if let Some(button) = item.button(mtm) {
        // Load icon from embedded PNG and set at 24pt
        let icon_bytes = include_bytes!("../icons/tray-icon.png");
        let ns_data = NSData::with_bytes(icon_bytes);
        if let Some(image) = NSImage::initWithData(NSImage::alloc(), &ns_data) {
            image.setSize(NSSize::new(28.6, 18.0));
            // Not template — we want color icon
            image.setTemplate(false);
            button.setImage(Some(&image));
        }

        // Set initial title
        button.setTitle(&NSString::from_str(title));

        // Wire up click handler via target-action
        let handler = TrayHandler::new();
        unsafe {
            button.setTarget(Some(&*handler));
            button.setAction(Some(sel!(handleClick:)));
        }
        // Keep handler alive for app lifetime
        std::mem::forget(handler);
    }

    // Store status item for later title updates (leaked intentionally)
    let ptr = Retained::into_raw(item) as *mut c_void;
    STATUS_ITEM_PTR.store(ptr, Ordering::Release);

    if cfg!(debug_assertions) {
        eprintln!("[52Hz] native NSStatusItem created (24pt icon)");
    }

    Ok(())
}

pub(crate) fn set_tray_icon_visible(visible: bool) {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let Some(item) = get_status_item() else {
        return;
    };
    if let Some(button) = item.button(mtm) {
        if visible {
            let icon_bytes = include_bytes!("../icons/tray-icon.png");
            let ns_data = NSData::with_bytes(icon_bytes);
            if let Some(image) = NSImage::initWithData(NSImage::alloc(), &ns_data) {
                image.setSize(NSSize::new(28.6, 18.0));
                image.setTemplate(false);
                button.setImage(Some(&image));
            }
        } else {
            button.setImage(None);
        }
    }
}

pub(crate) fn update_tray_title(title: &str) {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let Some(item) = get_status_item() else {
        return;
    };
    if let Some(button) = item.button(mtm) {
        button.setTitle(&NSString::from_str(title));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- calc_window_position ---

    #[test]
    fn window_centered_below_button() {
        // Button at x=900, Cocoa y=1420 (top of a 1440-screen), width=80
        // Window width=320
        let (x, y) = calc_window_position(900.0, 1420.0, 80.0, 1440.0, 320.0);
        // x: centered = 900 + 40 - 160 = 780
        assert!((x - 780.0).abs() < 0.01);
        // y: top-left = 1440 - 1420 = 20 (just below menu bar)
        assert!((y - 20.0).abs() < 0.01);
    }

    #[test]
    fn window_position_standard_menu_bar() {
        // Typical: 1440p screen, menu bar ~24pt, button frame y=1416 height=24
        let (x, y) = calc_window_position(1200.0, 1416.0, 60.0, 1440.0, 320.0);
        // x: 1200 + 30 - 160 = 1070
        assert!((x - 1070.0).abs() < 0.01);
        // y: 1440 - 1416 = 24
        assert!((y - 24.0).abs() < 0.01);
    }

    #[test]
    fn window_can_extend_left_of_screen() {
        // Button near left edge: x=10, width=40, window=320
        let (x, _y) = calc_window_position(10.0, 1420.0, 40.0, 1440.0, 320.0);
        // x: 10 + 20 - 160 = -130 (negative is OK, OS will clamp)
        assert!((x - (-130.0)).abs() < 0.01);
    }

    #[test]
    fn window_position_retina_screen() {
        // 2880x1800 logical (Retina), menu bar button near top
        let (x, y) = calc_window_position(2500.0, 1776.0, 80.0, 1800.0, 320.0);
        // x: 2500 + 40 - 160 = 2380
        assert!((x - 2380.0).abs() < 0.01);
        // y: 1800 - 1776 = 24
        assert!((y - 24.0).abs() < 0.01);
    }

    #[test]
    fn narrow_button_still_centers_window() {
        // Very narrow button (just an icon, no title)
        let (x, _y) = calc_window_position(500.0, 1420.0, 24.0, 1440.0, 320.0);
        // x: 500 + 12 - 160 = 352
        assert!((x - 352.0).abs() < 0.01);
    }

    #[test]
    fn wide_button_with_long_title() {
        // Wide button (icon + "20:00 (長い休憩)")
        let (x, _y) = calc_window_position(800.0, 1420.0, 200.0, 1440.0, 320.0);
        // x: 800 + 100 - 160 = 740
        assert!((x - 740.0).abs() < 0.01);
    }
}
