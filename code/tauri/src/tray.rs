use crate::{commands, SharedTimerState};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_positioner::{Position, WindowExt};

/// Update the tray menu's pause/resume label to match the current state.
pub(crate) fn sync_tray_pause_label(app: &tauri::AppHandle, paused: bool) {
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

pub(crate) fn build_tray(
    app: &tauri::App,
    title: &str,
    state: SharedTimerState,
) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    let pause_item = MenuItem::with_id(app, "toggle-pause", "Pause", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&pause_item, &quit_item])?;

    let _state = state; // keep for potential future use
    let tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("52Hz")
        .title(title)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle-pause" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app.try_state::<SharedTimerState>() {
                        commands::do_toggle_pause(&app, &state).await;
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
                            let mtm = unsafe { MainThreadMarker::new_unchecked() };
                            let ns_app = NSApplication::sharedApplication(mtm);
                            #[allow(deprecated)]
                            ns_app.activateIgnoringOtherApps(true);
                            if cfg!(debug_assertions) {
                                eprintln!("[52Hz] app activated for popup focus");
                            }
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
