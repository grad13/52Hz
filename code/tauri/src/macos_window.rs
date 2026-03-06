// meta: checked=2026-03-07
use tauri::Manager;

/// Apply native rounded corners to the main window via Titled style mask.
#[cfg(target_os = "macos")]
pub(super) fn setup_rounded_corners(window: &tauri::WebviewWindow) {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSWindow, NSWindowStyleMask};

    if let Ok(ns_window) = window.ns_window() {
        unsafe {
            let ns_win: Retained<NSWindow> =
                Retained::retain(ns_window as *mut NSWindow).unwrap();

            let mut mask = ns_win.styleMask();
            mask |= NSWindowStyleMask::Titled;
            mask |= NSWindowStyleMask::FullSizeContentView;
            ns_win.setStyleMask(mask);

            let _: () = objc2::msg_send![
                &*ns_win, setTitlebarAppearsTransparent: true
            ];
            let _: () = objc2::msg_send![
                &*ns_win, setTitleVisibility: 1_i64
            ];

            // Hide NSTitlebarContainerView
            if let Some(content_view) = ns_win.contentView() {
                let superview: *mut AnyObject =
                    objc2::msg_send![&*content_view, superview];
                if !superview.is_null() {
                    let subs: *mut AnyObject =
                        objc2::msg_send![superview, subviews];
                    let n: usize =
                        objc2::msg_send![subs, count];
                    for i in 0..n {
                        let sv: *mut AnyObject =
                            objc2::msg_send![
                                subs, objectAtIndex: i
                            ];
                        let cls: *mut AnyObject =
                            objc2::msg_send![sv, class];
                        let desc: *mut AnyObject =
                            objc2::msg_send![cls, description];
                        let cstr: *const std::ffi::c_char =
                            objc2::msg_send![
                                desc, UTF8String
                            ];
                        let name =
                            std::ffi::CStr::from_ptr(cstr)
                                .to_string_lossy();
                        if name.contains("Titlebar") {
                            let _: () = objc2::msg_send![
                                sv, setHidden: true
                            ];
                        }
                    }
                }
            }

            ns_win.setHasShadow(true);
        }
    }
}

/// Configure toast window NSWindow properties: level, transparency, WKWebView background, Spaces.
#[cfg(target_os = "macos")]
pub(super) fn setup_toast_transparency(
    toast_window: &tauri::WebviewWindow,
    app_handle: &tauri::AppHandle,
    level: &str,
) {
    use objc2::rc::Retained;
    use objc2_app_kit::{NSColor, NSWindow};

    if let Ok(ns_window) = toast_window.ns_window() {
        unsafe {
            let ns_win: Retained<NSWindow> =
                Retained::retain(ns_window as *mut NSWindow).unwrap();
            let win_level = if level == "back" { -1_isize } else { 25_isize };
            ns_win.setLevel(win_level);

            // Transparent window background
            ns_win.setOpaque(false);
            ns_win.setBackgroundColor(Some(&NSColor::clearColor()));
            ns_win.setHasShadow(false);

            // Disable WKWebView background drawing.
            // Delay so WKWebView is in the view hierarchy.
            let app_h = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let app_h2 = app_h.clone();
                let _ = app_h.run_on_main_thread(move || {
                    fn set_transparent(
                        view: *mut objc2::runtime::AnyObject,
                    ) {
                        unsafe {
                            let sel =
                                objc2::sel!(_setDrawsBackground:);
                            let responds: bool = objc2::msg_send![
                                view,
                                respondsToSelector: sel
                            ];
                            if responds {
                                let _: () = objc2::msg_send![
                                    view,
                                    _setDrawsBackground: false
                                ];
                            }
                            let subs: *mut objc2::runtime::AnyObject =
                                objc2::msg_send![view, subviews];
                            let n: usize =
                                objc2::msg_send![subs, count];
                            for i in 0..n {
                                let sv: *mut objc2::runtime::AnyObject
                                    = objc2::msg_send![
                                        subs,
                                        objectAtIndex: i
                                    ];
                                set_transparent(sv);
                            }
                        }
                    }
                    if let Some(tw) =
                        app_h2.get_webview_window("presence-toast")
                    {
                        if let Ok(ns_ptr) = tw.ns_window() {
                            let ns_w: objc2::rc::Retained<
                                objc2_app_kit::NSWindow,
                            > = objc2::rc::Retained::retain(
                                ns_ptr
                                    as *mut objc2_app_kit::NSWindow,
                            )
                            .unwrap();
                            if let Some(cv) = ns_w.contentView() {
                                let ptr: *mut objc2::runtime::AnyObject =
                                    objc2::msg_send![&*cv, self];
                                set_transparent(ptr);
                            }
                        }
                    }
                });
            });

            // Visible on all Spaces
            let _: () = objc2::msg_send![
                &*ns_win,
                setCollectionBehavior: 1_u64 | 16_u64
            ];
        }
    }
}

/// Register global click monitor for toast window first-click detection.
#[cfg(target_os = "macos")]
pub(super) fn setup_toast_click_monitor(app_handle: &tauri::AppHandle) {
    use objc2::rc::Retained;
    use objc2_app_kit::{NSEvent, NSEventMask, NSWindow};
    use tauri::Emitter;

    let app_for_toast_click = app_handle.clone();
    let toast_click_block =
        block2::RcBlock::new(move |_event: std::ptr::NonNull<NSEvent>| {
            if let Some(tw) =
                app_for_toast_click.get_webview_window("presence-toast")
            {
                if !tw.is_visible().unwrap_or(false) {
                    return;
                }
                unsafe {
                    let mouse_loc = NSEvent::mouseLocation();
                    if let Ok(ns_ptr) = tw.ns_window() {
                        let ns_w: Retained<NSWindow> =
                            Retained::retain(ns_ptr as *mut NSWindow).unwrap();
                        let frame = ns_w.frame();
                        if mouse_loc.x >= frame.origin.x
                            && mouse_loc.x
                                <= frame.origin.x + frame.size.width
                            && mouse_loc.y >= frame.origin.y
                            && mouse_loc.y
                                <= frame.origin.y + frame.size.height
                        {
                            let _ = app_for_toast_click
                                .emit("presence-toast-click", ());
                        }
                    }
                }
            }
        });
    let toast_monitor =
        NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
            NSEventMask::LeftMouseDown,
            &toast_click_block,
        );
    std::mem::forget(toast_monitor);
    std::mem::forget(toast_click_block);
}

/// Register global click monitor to hide main window on outside click.
#[cfg(target_os = "macos")]
pub(super) fn setup_outside_click_monitor(main_window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSEvent, NSEventMask};

    let w = main_window.clone();
    let block = block2::RcBlock::new(move |_event: std::ptr::NonNull<NSEvent>| {
        if w.is_visible().unwrap_or(false) {
            if cfg!(debug_assertions) {
                eprintln!("[52Hz] click outside detected → hiding main window");
            }
            let _ = w.hide();
        }
    });

    let monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
        NSEventMask::LeftMouseDown,
        &block,
    );
    // Keep monitor and block alive for app lifetime
    std::mem::forget(monitor);
    std::mem::forget(block);
}

/// Set toast window NSWindow level (front=25, back=-1).
#[cfg(target_os = "macos")]
pub(crate) fn set_toast_level(app_handle: &tauri::AppHandle, is_back: bool) {
    if let Some(tw) = app_handle.get_webview_window("presence-toast") {
        if let Ok(ns_ptr) = tw.ns_window() {
            unsafe {
                let ns_w: objc2::rc::Retained<objc2_app_kit::NSWindow> =
                    objc2::rc::Retained::retain(
                        ns_ptr as *mut objc2_app_kit::NSWindow,
                    )
                    .unwrap();
                let win_level: isize = if is_back { -1 } else { 25 };
                ns_w.setLevel(win_level);
            }
        }
    }
}
