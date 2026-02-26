use crate::timer::TimerSettings;
use crate::{create_break_overlay, sync_tray_pause_label, SharedTimerState};
use tauri::{Emitter, Manager};

pub(crate) async fn do_toggle_pause(app: &tauri::AppHandle, state: &SharedTimerState) -> bool {
    let mut s = state.lock().await;
    s.paused = !s.paused;
    let paused = s.paused;
    let state_clone = s.clone();
    drop(s);

    sync_tray_pause_label(app, paused);
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
pub(crate) async fn resume_timer(
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
pub(crate) async fn open_break_overlay(app: tauri::AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = create_break_overlay(&handle);
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn quit_app() {
    if cfg!(debug_assertions) {
        eprintln!("[RestRun] quit_app command invoked");
    }
    std::process::exit(0);
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
