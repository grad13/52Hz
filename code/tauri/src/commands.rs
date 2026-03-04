use crate::timer::{PhaseEvent, TimerSettings};
use crate::{overlay, SharedTimerState};
use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;

fn increment_today_sessions(app: &tauri::AppHandle) {
    let today = chrono::Local::now().format("sessions_%Y-%m-%d").to_string();
    if let Ok(store) = app.store("settings.json") {
        let current = store.get(&today).and_then(|v| v.as_u64()).unwrap_or(0);
        store.set(&today, serde_json::json!(current + 1));
    }
}

pub(crate) async fn do_toggle_pause(app: &tauri::AppHandle, state: &SharedTimerState) -> bool {
    let mut s = state.lock().await;
    s.paused = !s.paused;
    let paused = s.paused;
    let state_clone = s.clone();
    drop(s);

    let _ = app.emit("timer-tick", state_clone);
    paused
}

#[tauri::command]
pub(crate) async fn get_timer_state(
    state: tauri::State<'_, SharedTimerState>,
) -> Result<crate::timer::TimerState, String> {
    let s = state.lock().await;
    Ok(s.clone())
}

#[tauri::command]
pub(crate) async fn pause_timer(
    _app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.paused = true;
    drop(s);
    Ok(())
}

#[tauri::command]
pub(crate) async fn resume_timer(
    _app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.paused = false;
    drop(s);
    Ok(())
}

#[tauri::command]
pub(crate) async fn toggle_pause(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<bool, String> {
    let paused = do_toggle_pause(&app, &state).await;
    Ok(paused)
}

#[tauri::command]
pub(crate) async fn skip_break(
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
pub(crate) async fn update_settings(
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
pub(crate) async fn accept_break(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    let events = s.accept_break();
    let state_clone = s.clone();
    drop(s);

    for event in &events {
        match event {
            PhaseEvent::PhaseChanged => {
                let _ = app.emit("phase-changed", state_clone.clone());
            }
            PhaseEvent::BreakStart => {
                let _ = app.emit("break-start", state_clone.clone());
            }
            _ => {}
        }
    }
    let _ = app.emit("timer-tick", state_clone);
    increment_today_sessions(&app);
    Ok(())
}

#[tauri::command]
pub(crate) async fn extend_focus(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
    secs: u64,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.extend_focus(secs);
    let _ = app.emit("timer-tick", s.clone());
    Ok(())
}

#[tauri::command]
pub(crate) async fn skip_break_from_focus(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    let events = s.skip_break_from_focus();
    let state_clone = s.clone();
    drop(s);

    for event in &events {
        if let PhaseEvent::PhaseChanged = event {
            let _ = app.emit("phase-changed", state_clone.clone());
        }
    }
    let _ = app.emit("timer-tick", state_clone);
    increment_today_sessions(&app);
    Ok(())
}

#[tauri::command]
pub(crate) async fn get_today_sessions(app: tauri::AppHandle) -> Result<u64, String> {
    let today = chrono::Local::now().format("sessions_%Y-%m-%d").to_string();
    if let Ok(store) = app.store("settings.json") {
        Ok(store.get(&today).and_then(|v| v.as_u64()).unwrap_or(0))
    } else {
        Ok(0)
    }
}

#[tauri::command]
pub(crate) async fn open_break_overlay(app: tauri::AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = overlay::create_break_overlay(&handle);
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn reset_timer(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedTimerState>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.reset();
    let state_clone = s.clone();
    drop(s);
    let _ = app.emit("timer-tick", state_clone);
    Ok(())
}

#[tauri::command]
pub(crate) fn quit_app() {
    if cfg!(debug_assertions) {
        eprintln!("[52Hz] quit_app command invoked");
    }
    std::process::exit(0);
}

#[tauri::command]
pub(crate) async fn set_tray_icon_visible(app: tauri::AppHandle, visible: bool) -> Result<(), String> {
    app.run_on_main_thread(move || {
        crate::tray::set_tray_icon_visible(visible);
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn close_break_overlay(app: tauri::AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("break-overlay") {
            let _ = window.close();
        }
    })
    .map_err(|e| e.to_string())
}
