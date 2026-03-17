---
updated: 2026-03-15 09:02
checked: -
Retired: -
Format: spec-v2.1
Source: tauri/src/media.rs
---

# Specification: Media Control via AppleScript (media.rs)

## 0. Meta

| Source | Runtime |
|--------|---------|
| tauri/src/media.rs | Rust |

| Item | Value |
|------|-------|
| Related | lib.rs (called from break-start/break-end listeners) |
| Test Type | Unit (inline) |

## 1. Contract

```typescript
/** Constant array of supported media app names */
const MEDIA_APPS: string[];  // ["Spotify", "Music"]

/** Pause playing media apps and return list of paused app names (macOS only, pub) */
function pause_media_apps(): string[];

/** Resume playback of specified apps (macOS only, pub) */
function resume_media_apps(apps: string[]): void;

/** Generate AppleScript template for pause (internal) */
function build_pause_script(app_name: string): string;

/** Generate AppleScript template for resume (internal) */
function build_resume_script(app_name: string): string;

/** Execute AppleScript via osascript and return stdout (internal) */
function run_applescript(script: string): string | null;
```

## 2. State

This module holds no internal state.

## 3. Logic

### 3.1 Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MEDIA_APPS` | `["Spotify", "Music"]` | Media app names controlled via AppleScript |

### 3.2 build_pause_script

A pure function that takes an app name and generates a timeout-wrapped AppleScript for pausing.

Generated script structure:
```
if application "{app}" is running then
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
end if
```

- `is running` check avoids accessing apps that are not launched
- `with timeout 3 seconds` + `try` limits blocking to a maximum of 3 seconds when app is unresponsive
- Only executes `pause` and returns `"paused"` if the app is currently playing

### 3.3 build_resume_script

A pure function that takes an app name and generates a timeout-wrapped AppleScript for resuming.

Generated script structure:
```
if application "{app}" is running then
  try
    with timeout 3 seconds
      tell application "{app}"
        play
      end tell
    end timeout
  end try
end if
```

### 3.4 run_applescript

Executes a script via `osascript -e <script>`.

```
1. Command::new("osascript").arg("-e").arg(script).output()
2. Process spawn failure -> None
3. Non-zero exit status -> None
4. Success -> Return trimmed stdout string as Some
```

### 3.5 pause_media_apps

Executes `build_pause_script` -> `run_applescript` for each app in `MEDIA_APPS`.

```
paused = []
for app in MEDIA_APPS:
  script = build_pause_script(app)
  result = run_applescript(script)
  if result == "paused":
    paused.push(app)
return paused
```

### 3.6 resume_media_apps

Executes `build_resume_script` -> `run_applescript` for each app name in the argument list.

```
for app in apps:
  script = build_resume_script(app)
  run_applescript(script)
```

## 4. Side Effects

| Side Effect | Trigger | Verification Method |
|------------|---------|-------------------|
| osascript process spawn | `run_applescript()` invocation | Manual testing. Not testable in CI (directly affects OS media control) |
| Media app pause | `pause_media_apps()` | Manually verify that media is paused during playback |
| Media app play | `resume_media_apps()` | Manually verify that paused media resumes playback |

## 5. Notes

- `#[cfg(target_os = "macos")]` makes this macOS-only. On non-macOS builds, `pause_media_apps` / `resume_media_apps` do not exist.
- `pause` / `play` are idempotent. A `pause` on an already stopped app or `play` on an already playing app does nothing. This differs from CGEventPost toggle-based approaches.
- The `is running` check avoids accessing apps that are not launched. This prevents the issue of Apple Music being unintentionally launched.
- Test strategy: Template generation by `build_pause_script` / `build_resume_script` is verified via unit tests. `run_applescript` / `pause_media_apps` / `resume_media_apps` depend on osascript and are tested manually.
