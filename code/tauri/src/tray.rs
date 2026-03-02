use crate::SharedTimerState;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_positioner::{Position, WindowExt};

pub(crate) fn build_tray(
    app: &tauri::App,
    title: &str,
    _state: SharedTimerState,
) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    let tray = TrayIconBuilder::with_id("main-tray")
        .icon(tauri::include_image!("icons/tray-icon.png"))
        .icon_as_template(false)
        .tooltip("52Hz")
        .title(title)
        .show_menu_on_left_click(false)
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

                        #[cfg(target_os = "macos")]
                        {
                            use objc2::MainThreadMarker;
                            use objc2_app_kit::NSApplication;
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
