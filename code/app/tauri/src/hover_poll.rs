// meta: updated=2026-03-16 07:20 checked=2026-03-07
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Manager;

static HOVER_POLLING: AtomicBool = AtomicBool::new(false);

/// Poll cursor position every 50ms and inject synthetic hover events
/// into the WKWebView. Stops when the window becomes hidden.
pub(crate) fn start(app_handle: tauri::AppHandle) {
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
                unsafe { objc2::msg_send![objc2::class!(NSEvent), mouseLocation] };

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
