mod timer;

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_positioner::{Position, WindowExt};
use timer::{SharedTimerState, TimerSettings, TimerState};
use tokio::sync::Mutex;

#[tauri::command]
async fn get_timer_state(
    state: tauri::State<'_, SharedTimerState>,
) -> Result<TimerState, String> {
    let s = state.lock().await;
    Ok(s.clone())
}

#[tauri::command]
async fn pause_timer(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.paused = true;
    drop(s);
    sync_tray_pause_label(&app, true);
    Ok(())
}

#[tauri::command]
async fn resume_timer(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.paused = false;
    drop(s);
    sync_tray_pause_label(&app, false);
    Ok(())
}

#[tauri::command]
async fn toggle_pause(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<bool, String> {
    let mut s = state.lock().await;
    s.paused = !s.paused;
    let paused = s.paused;
    let state_clone = s.clone();
    drop(s);

    sync_tray_pause_label(&app, paused);
    let _ = app.emit("timer-tick", state_clone);
    Ok(paused)
}

#[tauri::command]
async fn skip_break(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    let events = s.skip_break();
    if !events.is_empty() {
        let _ = app.emit("break-end", ());
        let _ = app.emit("phase-changed", s.clone());
    }
    Ok(())
}

#[tauri::command]
async fn update_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
    settings: TimerSettings,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.apply_settings(settings);
    let _ = app.emit("timer-tick", s.clone());
    Ok(())
}

#[tauri::command]
async fn open_break_overlay(app: tauri::AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = create_break_overlay(&handle);
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app() {
    if cfg!(debug_assertions) {
        eprintln!("[RestRun] quit_app command invoked");
    }
    std::process::exit(0);
}

#[tauri::command]
async fn close_break_overlay(app: tauri::AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("break-overlay") {
            let _ = window.close();
        }
    })
    .map_err(|e| e.to_string())
}

/// Update the tray menu's pause/resume label to match the current state.
fn sync_tray_pause_label(app: &tauri::AppHandle, paused: bool) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = handle.tray_by_id("main-tray") {
            let label = if paused { "Resume" } else { "Pause" };
            let item = MenuItem::with_id(&handle, "toggle-pause", label, true, None::<&str>);
            if let Ok(item) = item {
                let quit = MenuItem::with_id(&handle, "quit", "Quit", true, None::<&str>);
                if let Ok(quit) = quit {
                    if let Ok(menu) = Menu::with_items(&handle, &[&item, &quit]) {
                        let _ = tray.set_menu(Some(menu));
                    }
                }
            }
        }
    });
}

fn create_break_overlay(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("break-overlay") {
        let _ = window.close();
    }

    let overlay = WebviewWindowBuilder::new(
        app,
        "break-overlay",
        WebviewUrl::App("index.html?view=break".into()),
    )
    .title("")
    .decorations(false)
    .skip_taskbar(true)
    .closable(false)
    .minimizable(false)
    .maximizable(false)
    .resizable(false)
    .focused(true)
    .background_color(tauri::window::Color(10, 10, 14, 255)) // #0a0a0e — prevent white flash
    .build()?;

    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Retained;
        use objc2_app_kit::{
            NSApplication, NSApplicationPresentationOptions, NSWindow,
            NSWindowCollectionBehavior,
        };
        use objc2::MainThreadMarker;

        if let Some(monitor) = overlay.current_monitor()? {
            let size = monitor.size();
            let pos = monitor.position();
            overlay.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: pos.x,
                y: pos.y,
            }))?;
            overlay.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: size.width,
                height: size.height,
            }))?;
        }

        let ns_window = overlay.ns_window().map_err(|e| format!("{e}"))?;
        // SAFETY: This function is called from run_on_main_thread, so we are on the main thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        unsafe {
            let ns_win: Retained<NSWindow> =
                Retained::retain(ns_window as *mut NSWindow).unwrap();
            ns_win.setLevel(1000);
            ns_win.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary,
            );
        }

        // Lock down the UI during breaks: hide Dock/menu bar and block
        // Cmd+Tab, Cmd+H, and the Force Quit dialog.
        let ns_app = NSApplication::sharedApplication(mtm);
        ns_app.setPresentationOptions(
            NSApplicationPresentationOptions::HideDock
                | NSApplicationPresentationOptions::HideMenuBar
                | NSApplicationPresentationOptions::DisableProcessSwitching
                | NSApplicationPresentationOptions::DisableHideApplication
                | NSApplicationPresentationOptions::DisableForceQuit,
        );
        if cfg!(debug_assertions) {
            eprintln!("[RestRun] presentation-options → locked");
        }
    }

    #[cfg(not(target_os = "macos"))]
    overlay.set_fullscreen(true)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let timer_settings = if std::env::var("RESTRUN_TEST_FAST_TIMER").is_ok() {
        eprintln!("[RestRun] Using fast timer settings for testing");
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

    eprintln!("[RestRun] Starting...");
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(timer_state.clone())
        .invoke_handler(tauri::generate_handler![
            get_timer_state,
            pause_timer,
            resume_timer,
            toggle_pause,
            skip_break,
            update_settings,
            open_break_overlay,
            close_break_overlay,
            quit_app,
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
            .title("RestRun")
            .inner_size(320.0, 420.0)
            .visible(false)
            .resizable(false)
            .decorations(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .build()?;

            // Build tray context menu
            let pause_item =
                MenuItem::with_id(app, "toggle-pause", "Pause", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&pause_item, &quit_item])?;

            // Build tray icon
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("RestRun")
                .title(&initial_tray_title)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle-pause" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(state) = app.try_state::<SharedTimerState>() {
                                let mut s = state.lock().await;
                                s.paused = !s.paused;
                                let paused = s.paused;
                                let state_clone = s.clone();
                                drop(s);

                                sync_tray_pause_label(&app, paused);
                                let _ = app.emit("timer-tick", state_clone);
                            }
                        });
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window
                                    .as_ref()
                                    .window()
                                    .move_window(Position::TrayBottomCenter);
                                let _ = window.show();
                                let _ = window.set_focus();

                                // macOS: explicitly activate the app so the window
                                // properly receives focus and Focused(false) fires
                                // when clicking outside.
                                #[cfg(target_os = "macos")]
                                {
                                    use objc2::MainThreadMarker;
                                    use objc2_app_kit::NSApplication;
                                    // Tray events are dispatched on the main thread.
                                    let mtm =
                                        unsafe { MainThreadMarker::new_unchecked() };
                                    let ns_app =
                                        NSApplication::sharedApplication(mtm);
                                    #[allow(deprecated)]
                                    ns_app.activateIgnoringOtherApps(true);
                                    if cfg!(debug_assertions) {
                                        eprintln!("[RestRun] app activated for popup focus");
                                    }
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            // Hide from Dock
            #[cfg(target_os = "macos")]
            let _ = app
                .handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Hide settings popup when it loses focus
            if let Some(main_window) = app.get_webview_window("main") {
                let w = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = w.hide();
                    }
                });
            }

            // Listen for break-start to open overlay (must run on main thread for UI ops)
            let app_handle = app.handle().clone();
            app.listen("break-start", move |_event| {
                if cfg!(debug_assertions) {
                    eprintln!("[RestRun] break-start → opening overlay");
                }
                let handle = app_handle.clone();
                let _ = app_handle.run_on_main_thread(move || {
                    let _ = create_break_overlay(&handle);
                });
            });

            // Listen for break-end to close overlay (must run on main thread for UI ops)
            let app_handle2 = app.handle().clone();
            app.listen("break-end", move |_event| {
                if cfg!(debug_assertions) {
                    eprintln!("[RestRun] break-end → closing overlay");
                }
                let handle = app_handle2.clone();
                let _ = app_handle2.run_on_main_thread(move || {
                    if let Some(window) = handle.get_webview_window("break-overlay") {
                        let _ = window.close();
                    }

                    // Restore Dock and menu bar visibility.
                    #[cfg(target_os = "macos")]
                    {
                        use objc2::MainThreadMarker;
                        use objc2_app_kit::{
                            NSApplication, NSApplicationPresentationOptions,
                        };
                        let mtm = unsafe { MainThreadMarker::new_unchecked() };
                        let ns_app = NSApplication::sharedApplication(mtm);
                        ns_app.setPresentationOptions(
                            NSApplicationPresentationOptions::Default,
                        );
                        if cfg!(debug_assertions) {
                            eprintln!("[RestRun] presentation-options → default");
                        }
                    }
                });
            });

            // Start the timer
            timer::spawn_timer(app.handle().clone(), timer_state.clone());

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
