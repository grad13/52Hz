use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub(crate) fn create_break_overlay(
    app: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    // Headless mode: emit logs but skip actual window creation.
    if std::env::var("RESTRUN_HEADLESS").is_ok() {
        if cfg!(debug_assertions) {
            eprintln!("[RestRun] presentation-options → locked");
        }
        return Ok(());
    }

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
        use objc2::MainThreadMarker;
        use objc2_app_kit::{
            NSApplication, NSApplicationPresentationOptions, NSWindow,
            NSWindowCollectionBehavior,
        };

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

pub(crate) fn unlock_presentation() {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::{NSApplication, NSApplicationPresentationOptions};
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let ns_app = NSApplication::sharedApplication(mtm);
        ns_app.setPresentationOptions(NSApplicationPresentationOptions::Default);
    }
    if cfg!(debug_assertions) {
        eprintln!("[RestRun] presentation-options → default");
    }
}
