use std::process::Command;

/// Supported media apps for AppleScript control.
const MEDIA_APPS: &[&str] = &["Spotify", "Music"];

fn run_applescript(script: &str) -> Option<String> {
    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn build_pause_script(app_name: &str) -> String {
    format!(
        r#"if application "{app}" is running then
  try
    with timeout 3 seconds
      tell application "{app}"
        if player state is playing then
          pause
          return "paused"
        end if
      end tell
    end timeout
  end try
end if"#,
        app = app_name
    )
}

fn build_resume_script(app_name: &str) -> String {
    format!(
        r#"if application "{app}" is running then
  try
    with timeout 3 seconds
      tell application "{app}"
        play
      end tell
    end timeout
  end try
end if"#,
        app = app_name
    )
}

/// Pause all running media apps and return the list of apps that were paused.
#[cfg(target_os = "macos")]
pub fn pause_media_apps() -> Vec<String> {
    let mut paused = Vec::new();
    for &app in MEDIA_APPS {
        let script = build_pause_script(app);
        if let Some(result) = run_applescript(&script) {
            if result == "paused" {
                paused.push(app.to_string());
                if cfg!(debug_assertions) {
                    eprintln!("[52Hz] media: paused {}", app);
                }
            }
        }
    }
    paused
}

/// Resume playback for the specified apps.
#[cfg(target_os = "macos")]
pub fn resume_media_apps(apps: &[String]) {
    for app in apps {
        let script = build_resume_script(app);
        run_applescript(&script);
        if cfg!(debug_assertions) {
            eprintln!("[52Hz] media: resumed {}", app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pause_script_spotify() {
        let script = build_pause_script("Spotify");
        assert!(script.contains(r#"if application "Spotify" is running"#));
        assert!(script.contains("with timeout 3 seconds"));
        assert!(script.contains("pause"));
        assert!(script.contains(r#"return "paused""#));
    }

    #[test]
    fn test_build_pause_script_music() {
        let script = build_pause_script("Music");
        assert!(script.contains(r#"if application "Music" is running"#));
        assert!(script.contains("pause"));
    }

    #[test]
    fn test_build_resume_script_spotify() {
        let script = build_resume_script("Spotify");
        assert!(script.contains(r#"if application "Spotify" is running"#));
        assert!(script.contains("play"));
    }

    #[test]
    fn test_build_resume_script_music() {
        let script = build_resume_script("Music");
        assert!(script.contains(r#"if application "Music" is running"#));
        assert!(script.contains("play"));
    }

    #[test]
    fn test_media_apps_contains_expected() {
        assert!(MEDIA_APPS.contains(&"Spotify"));
        assert!(MEDIA_APPS.contains(&"Music"));
    }

    #[test]
    fn test_build_pause_script_contains_timeout() {
        let script = build_pause_script("Spotify");
        assert!(script.contains("with timeout"));
        assert!(script.contains("try"));
        assert!(script.contains("end try"));
    }
}
