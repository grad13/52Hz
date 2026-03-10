// meta: checked=2026-03-07
use std::sync::Arc;

use tauri::{Emitter, Listener, Manager};

use crate::macos_window;
use crate::media;
use crate::overlay;
use crate::timer::PhaseEvent;
use crate::SharedTimerState;

/// Register all event listeners for the app lifecycle.
pub(super) fn register_listeners(app: &tauri::App, timer_state: SharedTimerState) {
    // Media pause tracking: stores which apps were paused by us
    let media_paused_apps: Arc<std::sync::Mutex<Vec<String>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let browser_toggled_apps: Arc<std::sync::Mutex<Vec<String>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));

    register_break_start(app, media_paused_apps.clone(), browser_toggled_apps.clone());
    register_break_end(app, media_paused_apps, browser_toggled_apps);
    register_focus_done(app, timer_state);
    register_presence_level_change(app);
    register_presence_reposition(app);
}

fn register_break_start(
    app: &tauri::App,
    media_paused_apps: Arc<std::sync::Mutex<Vec<String>>>,
    browser_toggled_apps: Arc<std::sync::Mutex<Vec<String>>>,
) {
    let app_handle = app.handle().clone();
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
                *media_paused_apps.lock().unwrap() = paused;

                // Also toggle browser media (YouTube etc.)
                let toggled = media::toggle_browser_media();
                *browser_toggled_apps.lock().unwrap() = toggled;
            }
        }

        let handle = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            let _ = overlay::create_break_overlay(&handle);
        });
    });
}

fn register_break_end(
    app: &tauri::App,
    media_paused_apps: Arc<std::sync::Mutex<Vec<String>>>,
    browser_toggled_apps: Arc<std::sync::Mutex<Vec<String>>>,
) {
    let app_handle = app.handle().clone();
    app.listen("break-end", move |_event| {
        if cfg!(debug_assertions) {
            eprintln!("[52Hz] break-end → closing overlay");
        }

        // Resume media if we paused it
        #[cfg(target_os = "macos")]
        {
            let apps: Vec<String> =
                std::mem::take(&mut *media_paused_apps.lock().unwrap());
            if !apps.is_empty() {
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] break-end → resuming media: {:?}", apps);
                }
                media::resume_media_apps(&apps);
            }

            // Resume browser media if we toggled it
            let browsers: Vec<String> =
                std::mem::take(&mut *browser_toggled_apps.lock().unwrap());
            if !browsers.is_empty() {
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] break-end → resuming browser media: {:?}", browsers);
                }
                media::toggle_browser_media();
            }
        }

        // Headless mode: no overlay was created — just emit the log.
        if std::env::var("FIFTYTWOHZ_HEADLESS").is_ok() {
            overlay::unlock_presentation();
            return;
        }

        let handle = app_handle.clone();
        let _ = app_handle.run_on_main_thread(move || {
            if let Some(window) = handle.get_webview_window("break-overlay") {
                let _ = window.close();
            }
            overlay::unlock_presentation();
        });
    });
}

fn register_focus_done(app: &tauri::App, timer_state: SharedTimerState) {
    let app_handle = app.handle().clone();
    app.listen("focus-done", move |_event| {
        if std::env::var("FIFTYTWOHZ_HEADLESS").is_ok() {
            // Headless: skip popup, auto-accept break
            if cfg!(debug_assertions) {
                eprintln!("[52Hz] focus-done-popup → skipped (headless)");
            }
            let state = timer_state.clone();
            let handle = app_handle.clone();
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

        // Normal mode: show focus-done as toast card
        if cfg!(debug_assertions) {
            eprintln!("[52Hz] focus-done → emitting toast");
        }
        let handle = app_handle.clone();
        let _ = handle.emit("focus-done-toast", ());
    });
}

fn register_presence_level_change(app: &tauri::App) {
    #[cfg(target_os = "macos")]
    {
        let app_handle = app.handle().clone();
        app.listen("presence-level-change", move |event| {
            let level = event.payload().trim_matches('"').to_string();
            let app_h = app_handle.clone();
            let _ = app_handle.run_on_main_thread(move || {
                macos_window::set_toast_level(&app_h, &level);
            });
        });
    }
}

fn register_presence_reposition(app: &tauri::App) {
    fn reposition_toast(app_h: &tauri::AppHandle, pos: &str) {
        if let Some(tw) = app_h.get_webview_window("presence-toast") {
            if let Some(monitor) = tw.primary_monitor().ok().flatten() {
                let scale = monitor.scale_factor();
                let phys = monitor.size();
                let lw = phys.width as f64 / scale;
                let lh = phys.height as f64 / scale;
                let size = tw.outer_size().unwrap_or(tauri::PhysicalSize::new(290, 80));
                let win_h = size.height as f64 / scale;
                let win_w = size.width as f64 / scale;
                let margin = 16.0;
                let margin_top = 40.0;
                let (x, y) = match pos {
                    "top-left" => (margin, margin_top),
                    "bottom-right" => (lw - win_w - margin, lh - win_h - margin),
                    "bottom-left" => (margin, lh - win_h - margin),
                    _ => (lw - win_w - margin, margin_top),
                };
                let _ = tw.set_position(tauri::LogicalPosition::new(x.max(0.0), y.max(0.0)));
            }
        }
    }

    // Reposition on every toast change (add/remove)
    let app_r1 = app.handle().clone();
    app.listen("presence-reposition", move |event| {
        let pos = event.payload().trim_matches('"').to_string();
        let app_h = app_r1.clone();
        let _ = app_r1.run_on_main_thread(move || {
            reposition_toast(&app_h, &pos);
        });
    });

    // Reposition on position setting change
    let app_r2 = app.handle().clone();
    app.listen("presence-position-change", move |event| {
        let pos = event.payload().trim_matches('"').to_string();
        let app_h = app_r2.clone();
        let _ = app_r2.run_on_main_thread(move || {
            reposition_toast(&app_h, &pos);
        });
    });
}
