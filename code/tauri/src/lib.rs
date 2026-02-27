mod commands;
mod overlay;
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
            let handle = app_handle.clone();
            let _ = app_handle.run_on_main_thread(move || {
                if let Some(tray) = handle.tray_by_id("main-tray") {
                    let _ = tray.set_title(Some(&title));
                }
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
        .manage(timer_state.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_timer_state,
            commands::pause_timer,
            commands::resume_timer,
            commands::toggle_pause,
            commands::skip_break,
            commands::update_settings,
            commands::open_break_overlay,
            commands::close_break_overlay,
            commands::quit_app,
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

            // Build tray icon with menu
            let _tray = tray::build_tray(app, &initial_tray_title, timer_state.clone())?;

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
                    eprintln!("[52Hz] break-start → opening overlay");
                }
                let handle = app_handle.clone();
                let _ = app_handle.run_on_main_thread(move || {
                    let _ = overlay::create_break_overlay(&handle);
                });
            });

            // Listen for break-end to close overlay (must run on main thread for UI ops)
            let app_handle2 = app.handle().clone();
            app.listen("break-end", move |_event| {
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] break-end → closing overlay");
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

            // Start the timer
            spawn_timer(app.handle().clone(), timer_state.clone());

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
