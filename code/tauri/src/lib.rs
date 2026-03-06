// meta: checked=2026-03-07
mod commands;
mod event_handlers;
mod hover_poll;
mod macos_window;
mod media;
mod overlay;
mod presence;
mod settings_loader;
mod timer;
mod tray;

use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
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
            let main_window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title("52Hz")
            .inner_size(320.0, 480.0)
            .visible(false)
            .resizable(false)
            .decorations(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .build()?;

            #[cfg(target_os = "macos")]
            macos_window::setup_rounded_corners(&main_window);

            // Build tray icon with menu
            tray::build_tray(app, &initial_tray_title)?;

            // Hide from Dock
            #[cfg(target_os = "macos")]
            let _ = app
                .handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Hide settings popup when clicking outside the app (macOS)
            #[cfg(target_os = "macos")]
            if let Some(main_window) = app.get_webview_window("main") {
                macos_window::setup_outside_click_monitor(&main_window);
            }

            // Register all event listeners
            event_handlers::register_listeners(app, timer_state.clone());

            // Load settings from store
            settings_loader::load_and_apply(app, &timer_state);
            let (initial_position, initial_level) = settings_loader::load_presence_settings(app);

            // Create toast window (transparent, mouse-through, always on top)
            {
                let toast_w = 290.0_f64;
                let toast_h = 80.0_f64;
                let margin = 16.0_f64;
                let margin_top = 40.0_f64; // clear macOS menu bar
                let (tx, ty) = if let Some(monitor) = app
                    .get_webview_window("main")
                    .and_then(|w| w.primary_monitor().ok().flatten())
                {
                    let scale = monitor.scale_factor();
                    let phys = monitor.size();
                    let lw = phys.width as f64 / scale;
                    let lh = phys.height as f64 / scale;
                    match initial_position.as_str() {
                        "top-left" => (margin, margin_top),
                        "bottom-right" => ((lw - toast_w - margin).max(0.0), (lh - toast_h - margin).max(0.0)),
                        "bottom-left" => (margin, (lh - toast_h - margin).max(0.0)),
                        _ => ((lw - toast_w - margin).max(0.0), margin_top), // top-right default
                    }
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

                #[cfg(target_os = "macos")]
                macos_window::setup_toast_transparency(
                    &toast_window,
                    app.handle(),
                    &initial_level,
                );
            }

            // Toast first-click monitor
            #[cfg(target_os = "macos")]
            macos_window::setup_toast_click_monitor(app.handle());

            // Start the timer
            spawn_timer(app.handle().clone(), timer_state.clone());

            // Start presence scheduler
            let chat_db = app
                .path()
                .resource_dir()
                .expect("resource_dir")
                .join("chat.db");
            presence::spawn(app.handle().clone(), chat_db);

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
