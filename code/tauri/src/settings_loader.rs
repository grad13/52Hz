// meta: checked=2026-03-07
use crate::timer::TimerSettings;
use crate::tray;
use crate::SharedTimerState;

/// Apply hide_tray_icon setting and load saved timer settings from store.
pub(super) fn load_and_apply(app: &tauri::App, timer_state: &SharedTimerState) {
    use tauri_plugin_store::StoreExt;

    // Apply hide_tray_icon setting
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

    // Load saved settings from store before starting the timer
    match app.store("settings.json") {
        Ok(store) => {
            let fm = store.get("focus_minutes").and_then(|v| v.as_f64());
            let sbs = store.get("short_break_minutes").and_then(|v| v.as_f64());
            let lbm = store.get("long_break_minutes").and_then(|v| v.as_f64());
            let sbbl = store.get("short_breaks_before_long").and_then(|v| v.as_f64());
            if fm.is_some() || sbs.is_some() || lbm.is_some() || sbbl.is_some() {
                let settings = TimerSettings {
                    focus_duration_secs: fm.map(|v| (v * 60.0) as u64).unwrap_or(1200),
                    short_break_duration_secs: sbs.map(|v| (v * 60.0) as u64).unwrap_or(60),
                    long_break_duration_secs: lbm.map(|v| (v * 60.0) as u64).unwrap_or(180),
                    short_breaks_before_long: sbbl.map(|v| v as u32).unwrap_or(3),
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

/// Load presence position and level from store.
pub(super) fn load_presence_settings(app: &tauri::App) -> (String, String) {
    use tauri_plugin_store::StoreExt;

    let pos = app
        .store("settings.json")
        .ok()
        .and_then(|s| s.get("presence_position"))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "top-right".into());
    let lvl = app
        .store("settings.json")
        .ok()
        .and_then(|s| s.get("presence_level"))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "front".into());
    (pos, lvl)
}
