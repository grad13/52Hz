use serial_test::serial;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const BINARY: &str = "hz52";
const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn build_binary() {
    let status = Command::new("cargo")
        .args(["build", "--bin", BINARY])
        .current_dir(PROJECT_ROOT)
        .status()
        .expect("failed to run cargo build");
    assert!(status.success(), "cargo build failed");
}

fn binary_path() -> String {
    format!("{}/target/debug/{}", PROJECT_ROOT, BINARY)
}

/// RAII guard that kills Vite on drop (even on panic).
struct ViteGuard(Child);

impl Drop for ViteGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
        let _ = Command::new("sh")
            .args(["-c", "lsof -ti :1420 | xargs kill 2>/dev/null"])
            .status();
    }
}

fn start_vite() -> ViteGuard {
    let project_root = std::path::Path::new(PROJECT_ROOT)
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
    let child = Command::new("npx")
        .args(["vite", "--port", "1420"])
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start vite");
    std::thread::sleep(Duration::from_secs(3));
    ViteGuard(child)
}

fn store_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .expect("no data dir")
        .join("com.hz52.app")
}

fn clear_settings_store() {
    let store_path = store_dir().join("settings.json");
    if store_path.exists() {
        let _ = std::fs::remove_file(&store_path);
        eprintln!("[test] Cleared store at {:?}", store_path);
    }
}

fn write_settings_store(json: &str) {
    let dir = store_dir();
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join("settings.json"), json).expect("failed to write store");
    eprintln!("[test] Wrote store: {}", json);
}

/// Run binary and assert it stays alive for `secs` seconds.
/// Returns the captured stderr output for further assertions.
fn run_and_check_alive(env_vars: &[(&str, &str)], test_name: &str, secs: u64) -> String {
    let mut cmd = Command::new(binary_path());
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd.env("FIFTYTWOHZ_HEADLESS", "1");
    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().unwrap_or_else(|e| {
        panic!("[{}] failed to start binary: {}", test_name, e);
    });

    let start = Instant::now();
    let check_duration = Duration::from_secs(secs);

    while start.elapsed() < check_duration {
        match child.try_wait() {
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(500));
            }
            Ok(Some(status)) => {
                let elapsed = start.elapsed();
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = String::new();
                        let _ = s.read_to_string(&mut buf);
                        buf
                    })
                    .unwrap_or_default();
                panic!(
                    "[{}] App crashed after {:.1}s with status {}.\n\
                     Expected: stay alive for at least {}s.\n\
                     Stderr:\n{}",
                    test_name,
                    elapsed.as_secs_f64(),
                    status,
                    secs,
                    stderr
                );
            }
            Err(e) => {
                panic!("[{}] Failed to check process status: {}", test_name, e);
            }
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    let stderr = child
        .stderr
        .take()
        .map(|mut s| {
            let mut buf = String::new();
            let _ = s.read_to_string(&mut buf);
            buf
        })
        .unwrap_or_default();

    eprintln!(
        "[{}] PASSED - app stayed alive for {}+ seconds",
        test_name, secs
    );
    stderr
}

/// Assert overlay lifecycle from stderr log.
fn assert_overlay_lifecycle(stderr: &str, min_opens: usize, min_closes: usize) {
    let overlays_opened = stderr.matches("opening overlay").count();
    let overlays_closed = stderr.matches("closing overlay").count();

    eprintln!(
        "[assert_overlay] opened={}, closed={}",
        overlays_opened, overlays_closed
    );

    assert!(
        overlays_opened >= min_opens,
        "Expected at least {} overlay opens, got {}.\nStderr:\n{}",
        min_opens,
        overlays_opened,
        stderr
    );

    assert!(
        overlays_closed >= min_closes,
        "Expected at least {} overlay closes, got {}.\nStderr:\n{}",
        min_closes,
        overlays_closed,
        stderr
    );

    // At most 1 unmatched open (the last break might be in-progress at kill time).
    assert!(
        overlays_opened <= overlays_closed + 1,
        "Too many unmatched overlays: opened={}, closed={}.\nStderr:\n{}",
        overlays_opened,
        overlays_closed,
        stderr
    );
}

/// Assert presentation options are locked during breaks and restored after.
fn assert_presentation_options_lifecycle(stderr: &str, min_locks: usize, min_restores: usize) {
    let locked = stderr.matches("presentation-options → locked").count();
    let restored = stderr.matches("presentation-options → default").count();

    eprintln!(
        "[assert_presentation] locked={}, restored={}",
        locked, restored
    );

    assert!(
        locked >= min_locks,
        "Expected at least {} presentation-options locks, got {}.\nStderr:\n{}",
        min_locks,
        locked,
        stderr
    );

    assert!(
        restored >= min_restores,
        "Expected at least {} presentation-options restores, got {}.\nStderr:\n{}",
        min_restores,
        restored,
        stderr
    );

    // Each lock should be followed by a restore (within 1 for in-progress break at kill time).
    assert!(
        locked <= restored + 1,
        "Too many unmatched presentation-options locks: locked={}, restored={}.\nStderr:\n{}",
        locked,
        restored,
        stderr
    );
}

/// Assert that phase transitions log the correct durations for fast timer
/// (focus=5s, short=3s, long=5s).
fn assert_fast_timer_durations(stderr: &str) {
    for line in stderr.lines() {
        if line.contains("phase-changed") {
            if line.contains("ShortBreak") {
                assert!(
                    line.contains("duration=3s"),
                    "ShortBreak should have duration=3s, got: {}",
                    line
                );
            } else if line.contains("LongBreak") {
                assert!(
                    line.contains("duration=5s"),
                    "LongBreak should have duration=5s, got: {}",
                    line
                );
            } else if line.contains("Focus") {
                assert!(
                    line.contains("duration=5s"),
                    "Focus should have duration=5s, got: {}",
                    line
                );
            }
        }
    }
}

// ---------------------------------------------------------------
// Tests: Basic survival
// ---------------------------------------------------------------

/// App must survive 30 seconds without dev server (no WebView content).
#[test]
#[serial]
fn app_survives_30s_without_devserver() {
    build_binary();
    run_and_check_alive(&[], "no_devserver_30s", 30);
}

/// App must survive 30 seconds WITH dev server (WebView loads real content).
#[test]
#[serial]
fn app_survives_30s_with_devserver() {
    build_binary();
    let _vite = start_vite();
    run_and_check_alive(&[], "with_devserver_30s", 30);
}

// ---------------------------------------------------------------
// Tests: Phase transitions with Vite
// ---------------------------------------------------------------

/// App must survive 90 seconds with fast timer AND go through multiple
/// phase transitions with overlay open/close.
///
/// Fast timer: Focus(5s) → Short(3s) → Focus(5s) → Long(5s) → ...
/// Full half-cycle (Focus + break) = 8s. In 90s → ~10 overlay opens.
///
/// Asserts:
///   1. Process stays alive for 90 seconds
///   2. At least 5 overlays opened
///   3. At least 5 overlays closed
///   4. Open/close counts match (within 1)
///   5. Phase durations match fast timer settings
#[test]
#[serial]
fn fast_timer_90s_phase_transitions_with_vite() {
    build_binary();
    clear_settings_store();
    let _vite = start_vite();

    let stderr = run_and_check_alive(
        &[("FIFTYTWOHZ_TEST_FAST_TIMER", "1")],
        "fast_timer_90s_vite",
        90,
    );

    assert_overlay_lifecycle(&stderr, 5, 5);
    assert_fast_timer_durations(&stderr);
    assert_presentation_options_lifecycle(&stderr, 5, 5);
}

// ---------------------------------------------------------------
// Tests: Phase transitions WITHOUT Vite
// ---------------------------------------------------------------

/// Fast timer phase transitions without Vite (no WebView content).
/// Verifies that the timer logic works independently of the frontend.
#[test]
#[serial]
fn fast_timer_60s_phase_transitions_without_vite() {
    build_binary();
    clear_settings_store();

    let stderr = run_and_check_alive(
        &[("FIFTYTWOHZ_TEST_FAST_TIMER", "1")],
        "fast_timer_60s_no_vite",
        60,
    );

    let phase_changes = stderr.matches("phase-changed").count();
    eprintln!(
        "[fast_timer_no_vite] phase-changed events: {}",
        phase_changes
    );
    assert!(
        phase_changes >= 5,
        "Expected at least 5 phase-changed events without Vite, got {}.\nStderr:\n{}",
        phase_changes,
        stderr
    );
}

// ---------------------------------------------------------------
// Tests: Store interaction
// ---------------------------------------------------------------

/// When the store is empty, fast timer env-var settings are not overridden.
/// All logged phase durations must match the env-var values (5s/3s/5s).
#[test]
#[serial]
fn store_empty_fast_timer_durations_correct() {
    build_binary();
    clear_settings_store();
    let _vite = start_vite();

    let stderr = run_and_check_alive(
        &[("FIFTYTWOHZ_TEST_FAST_TIMER", "1")],
        "store_empty_durations",
        30,
    );

    assert_fast_timer_durations(&stderr);

    let phase_changes = stderr.matches("phase-changed").count();
    assert!(
        phase_changes >= 1,
        "No phase transitions in 30s with fast timer.\nStderr:\n{}",
        stderr
    );
}

/// When the store has saved values, those values override the backend
/// defaults (including fast timer env-var). This is the designed behavior
/// for settings persistence.
///
/// We write a store with focus_minutes=1 (60s), then run with fast timer.
/// The frontend should push stored values, overriding focus from 5s to 60s.
/// In 30 seconds with focus=60s, we should see 0 phase transitions
/// (since 60s focus hasn't elapsed yet).
#[test]
#[serial]
fn store_values_override_fast_timer() {
    build_binary();
    write_settings_store(
        r#"{"focus_minutes":1,"short_break_secs":3,"long_break_minutes":1,"short_breaks_before_long":2}"#,
    );
    let _vite = start_vite();

    let stderr = run_and_check_alive(
        &[("FIFTYTWOHZ_TEST_FAST_TIMER", "1")],
        "store_overrides_fast_timer",
        30,
    );

    // With stored focus_minutes=1 (60s), no transition should occur in 30s.
    let phase_changes = stderr.matches("phase-changed").count();
    eprintln!(
        "[store_override] phase-changed events: {}",
        phase_changes
    );
    assert_eq!(
        phase_changes, 0,
        "Expected 0 phase transitions (focus=60s from store, only ran 30s), got {}.\n\
         This means store values are NOT overriding the backend.\nStderr:\n{}",
        phase_changes, stderr
    );

    // Cleanup: remove the test store so it doesn't affect other tests.
    clear_settings_store();
}

/// When the store has saved values and we DON'T use fast timer,
/// the stored settings should be applied. With focus_minutes=1 (60s)
/// and a 90-second run, we should see at least 1 transition.
#[test]
#[serial]
fn store_values_applied_on_normal_startup() {
    build_binary();
    write_settings_store(
        r#"{"focus_minutes":1,"short_break_secs":5,"long_break_minutes":1,"short_breaks_before_long":2}"#,
    );
    let _vite = start_vite();

    let stderr = run_and_check_alive(&[], "store_applied_normal", 90);

    // focus=60s, so first transition at ~60s. In 90s we should see >= 1 transition.
    let phase_changes = stderr.matches("phase-changed").count();
    eprintln!(
        "[store_applied] phase-changed events: {}",
        phase_changes
    );
    assert!(
        phase_changes >= 1,
        "Expected at least 1 phase transition (focus=60s from store, ran 90s), got {}.\n\
         Store settings may not be loading correctly.\nStderr:\n{}",
        phase_changes, stderr
    );

    clear_settings_store();
}
