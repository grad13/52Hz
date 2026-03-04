mod commands;
mod media;
mod overlay;
mod presence;
mod timer;
mod tray;

use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};
use timer::{PhaseEvent, TimerSettings, TimerState};
use tokio::sync::Mutex;

pub type SharedTimerState = Arc<Mutex<TimerState>>;

fn spawn_timer(app_handle: tauri::AppHandle, state: SharedTimerState) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        // tokio::time::interval fires immediately on the first tick.
        // Skip it so the first real tick happens after 1 second.
        interval.tick().await;
        loop {
            interval.tick().await;

            let mut s = state.lock().await;

            // Step 1: Advance the timer (increment elapsed).
            s.advance();

            // Step 2: Emit the current state BEFORE checking for transitions.
            // This ensures the frontend sees remaining=0 before the phase changes.
            let title = s.tray_title();
            let _ = app_handle.run_on_main_thread(move || {
                tray::update_tray_title(&title);
            });
            let _ = app_handle.emit("timer-tick", s.clone());

            // Step 3: Check for phase transition.
            let events = s.try_transition();

            // Step 4: Emit phase events (and a second timer-tick with the new state).
            if !events.is_empty() {
                let _ = app_handle.emit("timer-tick", s.clone());
            }
            for event in &events {
                match event {
                    PhaseEvent::PhaseChanged => {
                        if cfg!(debug_assertions) {
                            eprintln!(
                                "[52Hz] phase-changed → {:?} (duration={}s)",
                                s.phase, s.phase_duration_secs
                            );
                        }
                        let _ = app_handle.emit("phase-changed", s.clone());
                    }
                    PhaseEvent::BreakStart => {
                        if cfg!(debug_assertions) {
                            eprintln!("[52Hz] break-start → {:?}", s.phase);
                        }
                        let _ = app_handle.emit("break-start", s.clone());
                    }
                    PhaseEvent::BreakEnd => {
                        if cfg!(debug_assertions) {
                            eprintln!("[52Hz] break-end → back to Focus");
                        }
                        let _ = app_handle.emit("break-end", ());
                    }
                    PhaseEvent::FocusDone => {
                        if cfg!(debug_assertions) {
                            eprintln!("[52Hz] focus-done → paused, awaiting user choice");
                        }
                        // Always emit focus-done so the listener handles it
                        // (both HEADLESS and normal mode).
                        let _ = app_handle.emit("focus-done", s.clone());
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let timer_settings = if std::env::var("FIFTYTWOHZ_TEST_FAST_TIMER").is_ok() {
        eprintln!("[52Hz] Using fast timer settings for testing");
        TimerSettings {
            focus_duration_secs: 5,
            short_break_duration_secs: 3,
            long_break_duration_secs: 5,
            short_breaks_before_long: 2,
        }
    } else {
        TimerSettings::default()
    };
    let timer_state: SharedTimerState = Arc::new(Mutex::new(TimerState::new(timer_settings)));

    let initial_tray_title = {
        let s = timer_state.try_lock().unwrap();
        s.tray_title()
    };

    eprintln!("[52Hz] Starting...");
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(timer_state.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_timer_state,
            commands::pause_timer,
            commands::resume_timer,
            commands::toggle_pause,
            commands::skip_break,
            commands::update_settings,
            commands::accept_break,
            commands::extend_focus,
            commands::skip_break_from_focus,
            commands::get_today_sessions,
            commands::open_break_overlay,
            commands::close_break_overlay,
            commands::set_tray_icon_visible,
            commands::reset_timer,
            commands::quit_app,
            commands::test_presence_toast,
        ])
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Create main window (hidden by default)
            let _main_window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title("52Hz")
            .inner_size(320.0, 420.0)
            .visible(false)
            .resizable(false)
            .decorations(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .build()?;

            // Native rounded corners via Titled style mask
            #[cfg(target_os = "macos")]
            {
                use objc2::rc::Retained;
                use objc2::runtime::AnyObject;
                use objc2_app_kit::{NSWindow, NSWindowStyleMask};

                if let Ok(ns_window) = _main_window.ns_window() {
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

            // Build tray icon with menu
            tray::build_tray(app, &initial_tray_title)?;

            // Hide from Dock
            #[cfg(target_os = "macos")]
            let _ = app
                .handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Hide settings popup when clicking outside the app (macOS)
            // ActivationPolicy::Accessory prevents normal Focused(false) events,
            // so we use NSEvent global monitor to detect clicks outside our app.
            #[cfg(target_os = "macos")]
            if let Some(main_window) = app.get_webview_window("main") {
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

            // Media pause tracking: stores which apps were paused by us
            let media_paused_apps: Arc<std::sync::Mutex<Vec<String>>> =
                Arc::new(std::sync::Mutex::new(Vec::new()));

            // Listen for break-start to open overlay (must run on main thread for UI ops)
            let app_handle = app.handle().clone();
            let media_apps_start = media_paused_apps.clone();
            app.listen("break-start", move |_event| {
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] break-start → opening overlay");
                }

                // Pause media if setting is enabled
                #[cfg(target_os = "macos")]
                {
                    use tauri_plugin_store::StoreExt;
                    let should_pause = app_handle
                        .store("settings.json")
                        .ok()
                        .and_then(|s| s.get("pause_media_on_break"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if should_pause {
                        if cfg!(debug_assertions) {
                            eprintln!("[52Hz] break-start → pausing media");
                        }
                        let paused = media::pause_media_apps();
                        *media_apps_start.lock().unwrap() = paused;
                    }
                }

                let handle = app_handle.clone();
                let _ = app_handle.run_on_main_thread(move || {
                    let _ = overlay::create_break_overlay(&handle);
                });
            });

            // Listen for break-end to close overlay (must run on main thread for UI ops)
            let app_handle2 = app.handle().clone();
            let media_apps_end = media_paused_apps.clone();
            app.listen("break-end", move |_event| {
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] break-end → closing overlay");
                }

                // Resume media if we paused it
                #[cfg(target_os = "macos")]
                {
                    let apps: Vec<String> =
                        std::mem::take(&mut *media_apps_end.lock().unwrap());
                    if !apps.is_empty() {
                        if cfg!(debug_assertions) {
                            eprintln!("[52Hz] break-end → resuming media: {:?}", apps);
                        }
                        media::resume_media_apps(&apps);
                    }
                }

                // Headless mode: no overlay was created — just emit the log.
                if std::env::var("FIFTYTWOHZ_HEADLESS").is_ok() {
                    overlay::unlock_presentation();
                    return;
                }

                let handle = app_handle2.clone();
                let _ = app_handle2.run_on_main_thread(move || {
                    if let Some(window) = handle.get_webview_window("break-overlay") {
                        let _ = window.close();
                    }
                    overlay::unlock_presentation();
                });
            });

            // Listen for focus-done: open popup (normal) or auto-accept (headless)
            let app_handle3 = app.handle().clone();
            let focus_done_state = timer_state.clone();
            app.listen("focus-done", move |_event| {
                if std::env::var("FIFTYTWOHZ_HEADLESS").is_ok() {
                    // Headless: skip popup, auto-accept break
                    eprintln!("[52Hz] focus-done-popup → skipped (headless)");
                    let state = focus_done_state.clone();
                    let handle = app_handle3.clone();
                    tauri::async_runtime::spawn(async move {
                        let mut s = state.lock().await;
                        let events = s.accept_break();
                        let state_clone = s.clone();
                        drop(s);
                        for event in &events {
                            match event {
                                PhaseEvent::PhaseChanged => {
                                    if cfg!(debug_assertions) {
                                        eprintln!(
                                            "[52Hz] phase-changed → {:?} (duration={}s)",
                                            state_clone.phase, state_clone.phase_duration_secs
                                        );
                                    }
                                    let _ = handle.emit("phase-changed", state_clone.clone());
                                }
                                PhaseEvent::BreakStart => {
                                    if cfg!(debug_assertions) {
                                        eprintln!(
                                            "[52Hz] break-start → {:?}",
                                            state_clone.phase
                                        );
                                    }
                                    let _ = handle.emit("break-start", state_clone.clone());
                                }
                                _ => {}
                            }
                        }
                        let _ = handle.emit("timer-tick", state_clone);
                    });
                    return;
                }

                // Normal mode: open focus-done popup
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] focus-done-popup → creating");
                }
                let handle = app_handle3.clone();
                let _ = app_handle3.run_on_main_thread(move || {
                    // Close existing popup if any
                    if let Some(w) = handle.get_webview_window("focus-done-popup") {
                        let _ = w.close();
                    }
                    // macOS: activate app BEFORE showing window
                    // (Accessory policy apps need explicit activation)
                    #[cfg(target_os = "macos")]
                    {
                        use objc2::MainThreadMarker;
                        use objc2_app_kit::NSApplication;
                        let mtm = unsafe { MainThreadMarker::new_unchecked() };
                        let ns_app = NSApplication::sharedApplication(mtm);
                        #[allow(deprecated)]
                        ns_app.activateIgnoringOtherApps(true);
                    }

                    let popup_w = 300.0_f64;
                    let popup_h = 200.0_f64;
                    let margin = 20.0_f64;

                    let (x, y) = if let Some(monitor) = handle
                        .get_webview_window("main")
                        .and_then(|w| w.primary_monitor().ok().flatten())
                    {
                        let scale = monitor.scale_factor();
                        let phys = monitor.size();
                        let lw = phys.width as f64 / scale;
                        let lh = phys.height as f64 / scale;
                        (
                            (lw - popup_w - margin).max(0.0),
                            (lh * 0.05).max(margin), // 画面上端 5% の位置
                        )
                    } else {
                        (margin, margin)
                    };

                    match WebviewWindowBuilder::new(
                        &handle,
                        "focus-done-popup",
                        WebviewUrl::App("index.html?view=focus-done".into()),
                    )
                    .title("Focus Complete")
                    .inner_size(popup_w, popup_h)
                    .position(x, y)
                    .visible(false) // create hidden, then show after setting level
                    .resizable(false)
                    .decorations(false)
                    .skip_taskbar(true)
                    .always_on_top(true)
                    .focused(true)
                    .build()
                    {
                        Ok(window) => {
                            #[cfg(target_os = "macos")]
                            {
                                use objc2::rc::Retained;
                                use objc2_app_kit::NSWindow;
                                if let Ok(ns_window) = window.ns_window() {
                                    unsafe {
                                        let ns_win: Retained<NSWindow> =
                                            Retained::retain(ns_window as *mut NSWindow)
                                                .unwrap();
                                        ns_win.setLevel(25); // NSStatusWindowLevel
                                        ns_win.makeKeyAndOrderFront(None);
                                    }
                                }
                            }
                            let _ = window.show();
                            let _ = window.set_focus();
                            eprintln!("[52Hz] focus-done-popup → created");
                        }
                        Err(e) => {
                            eprintln!("[52Hz] focus-done-popup → FAILED: {}", e);
                        }
                    }
                });
            });

            // Apply hide_tray_icon setting
            {
                use tauri_plugin_store::StoreExt;
                let hide_icon = app
                    .store("settings.json")
                    .ok()
                    .and_then(|s| s.get("hide_tray_icon"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if hide_icon {
                    tray::set_tray_icon_visible(false);
                    if cfg!(debug_assertions) {
                        eprintln!("[52Hz] tray icon hidden (user setting)");
                    }
                }
            }

            // Load saved settings from store before starting the timer
            {
                use tauri_plugin_store::StoreExt;
                match app.store("settings.json") {
                    Ok(store) => {
                        let fm = store.get("focus_minutes").and_then(|v| v.as_f64());
                        let sbs = store.get("short_break_minutes").and_then(|v| v.as_f64());
                        let lbm = store.get("long_break_minutes").and_then(|v| v.as_f64());
                        let sbbl = store.get("short_breaks_before_long").and_then(|v| v.as_f64());
                        if fm.is_some() || sbs.is_some() || lbm.is_some() || sbbl.is_some() {
                            let settings = TimerSettings {
                                focus_duration_secs: fm
                                    .map(|v| (v * 60.0) as u64)
                                    .unwrap_or(1200),
                                short_break_duration_secs: sbs
                                    .map(|v| (v * 60.0) as u64)
                                    .unwrap_or(60),
                                long_break_duration_secs: lbm
                                    .map(|v| (v * 60.0) as u64)
                                    .unwrap_or(180),
                                short_breaks_before_long: sbbl
                                    .map(|v| v as u32)
                                    .unwrap_or(3),
                            };
                            timer_state.try_lock().unwrap().apply_settings(settings);
                            eprintln!("[52Hz] Loaded saved settings from store");
                        }
                    }
                    Err(_) => {
                        eprintln!("[52Hz] No settings store found, using defaults");
                    }
                }
            }

            // Create toast window (transparent, mouse-through, always on top)
            {
                let toast_w = 290.0_f64;
                let toast_h = 80.0_f64;
                let margin = 16.0_f64;
                let (tx, ty) = if let Some(monitor) = app
                    .get_webview_window("main")
                    .and_then(|w| w.primary_monitor().ok().flatten())
                {
                    let scale = monitor.scale_factor();
                    let phys = monitor.size();
                    let lw = phys.width as f64 / scale;
                    ((lw - toast_w - margin).max(0.0), margin)
                } else {
                    (margin, margin)
                };

                let toast_window = WebviewWindowBuilder::new(
                    app,
                    "presence-toast",
                    WebviewUrl::App("index.html?view=toast".into()),
                )
                .title("")
                .inner_size(toast_w, toast_h)
                .position(tx, ty)
                .visible(false) // hidden until a message arrives
                .resizable(false)
                .decorations(false)
                .skip_taskbar(true)
                .always_on_top(true)
                .build()?;

                // macOS: mouse-through, float above everything, rounded corners
                #[cfg(target_os = "macos")]
                {
                    use objc2::rc::Retained;
                    use objc2_app_kit::{NSWindow, NSWindowStyleMask};
                    if let Ok(ns_window) = toast_window.ns_window() {
                        unsafe {
                            let ns_win: Retained<NSWindow> =
                                Retained::retain(ns_window as *mut NSWindow).unwrap();
                            ns_win.setIgnoresMouseEvents(true);
                            ns_win.setLevel(25); // NSStatusWindowLevel

                            // Rounded corners via Titled mask
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

                            // Visible on all Spaces
                            let _: () = objc2::msg_send![
                                &*ns_win,
                                setCollectionBehavior: 1_u64 | 16_u64
                            ];
                        }
                    }
                }
                let _ = toast_window;
            }

            // Start the timer
            spawn_timer(app.handle().clone(), timer_state.clone());

            // Start presence scheduler
            presence::spawn(app.handle().clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
