---
updated: 2026-05-08
checked: -
Retired: -
Format: spec-v2.1
Source: tauri/src/media.rs
---

# Specification: Media Control via MediaRemote.framework (media.rs)

## 0. Meta

| Source | Runtime |
|--------|---------|
| tauri/src/media.rs | Rust |

| Item | Value |
|------|-------|
| Related | event_handlers.rs (called from break-start / break-end listeners) |
| Test Type | Unit (inline) |

## 1. Contract

```typescript
/** Send Now Playing toggle (play↔pause) to whichever app currently owns
 *  Now Playing. macOS only. Pub. Panics on failure. */
function post_play_pause_key(): void;
```

No other public symbols are exported. Internal helpers (`last_dl_error`,
`dlopen` / `dlsym` externs) are private.

## 2. State

This module holds no internal state.

## 3. Logic

### 3.1 Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MEDIA_REMOTE_PATH` | `"/System/Library/PrivateFrameworks/MediaRemote.framework/MediaRemote"` | Absolute path to the MediaRemote framework binary, loaded via dlopen at call time |
| `MR_TOGGLE_PLAY_PAUSE` | `2` | `kMRTogglePlayPause` command identifier passed to `MRMediaRemoteSendCommand` (recovered from `MediaRemote.h` class-dump) |
| `RTLD_NOW` | `2` | dlopen flag — resolve all symbols at load time so a missing symbol surfaces immediately |

### 3.2 post_play_pause_key

The single public entry point. Sends one Play/Pause toggle to the current
Now Playing application via the private `MRMediaRemoteSendCommand` symbol.

Procedure:

```
1. dlopen(MEDIA_REMOTE_PATH, RTLD_NOW)
   - On null handle: panic with the message from dlerror()
2. dlsym(handle, "MRMediaRemoteSendCommand")
   - On null pointer: panic with the message from dlerror()
3. Cast the resolved pointer to:
     unsafe extern "C" fn(u32, *mut c_void) -> bool
4. Call send_command(MR_TOGGLE_PLAY_PAUSE, NULL)
5. log::info! the boolean return value
```

The dynamically-loaded handle is intentionally **not** dlclose'd: keeping
the framework resident across breaks avoids the cost of repeatedly mapping
it. (Process exit reclaims everything.)

### 3.3 last_dl_error (internal)

Reads the most recent dlerror() string and returns it as an owned `String`.
Returns an empty string when dlerror() yields a null pointer.

```
1. p = dlerror()
2. if p == NULL: return ""
3. else: return CStr::from_ptr(p).to_string_lossy().into_owned()
```

## 4. Side Effects

| Side Effect | Trigger | Verification Method |
|------------|---------|-------------------|
| dlopen of MediaRemote.framework | Each `post_play_pause_key` call | Manual; confirmed by absence of dlopen panic in logs |
| Now Playing toggle (play↔pause) | `MRMediaRemoteSendCommand` invocation | Manual: play media in any app (Music, Spotify, Firefox/YouTube, Podcasts, …), trigger break, verify it pauses; verify it resumes at break-end |
| log::info entry | Every successful invocation | Tail `~/Library/Logs/com.hz52.app/52Hz.log` for `media: MRMediaRemoteSendCommand(TogglePlayPause) → <bool>` |

## 5. Notes

### 5.1 Why MediaRemote, not CGEvent / AppleScript

- **Equivalent to the F8 media key** — both target whichever app holds Now
  Playing. The user-facing semantics match what pressing the physical key
  does.
- **No Accessibility entitlement required.** Synthesizing a CGEvent for the
  Play/Pause key would be classified as input injection and trigger the
  Accessibility prompt; MediaRemote is a media-control API and bypasses it.
- **Single mechanism — no fallbacks.** The earlier design combined Spotify /
  Music AppleScript, browser `'k'`-key injection, and a CGEvent fallback;
  each path could fail silently and resume paths could leak state. The
  `'k'` injection also stole focus from the break overlay. All of that is
  gone.
- **Coverage.** Music / Spotify / Podcasts and any HTML5 `<video>` /
  `<audio>` element registered with the Media Session API in any browser
  (Firefox, Safari, Chrome, Arc, Edge, …) are all reachable.

### 5.2 Toggle semantics

`MR_TOGGLE_PLAY_PAUSE` is a *toggle*: playing ↔ paused. The break listener
calls `post_play_pause_key` once at break-start (pause) and once at
break-end (play). If nothing was playing at break-start, the second call
will start playback of the most recent Now Playing source — this matches
the behavior of pressing F8 twice and is intentional.

### 5.3 Failure mode

`post_play_pause_key` panics — does not return Result — when dlopen or
dlsym fails. This is the explicit contract: a silent no-op is the worst
outcome (the user sees the break overlay but media keeps playing, with
no observable error). A panic surfaces the bug instead. The dynamically
loaded MediaRemote framework is a private API; if Apple removes or renames
the symbol in a future macOS, the panic message points at the exact
failure (`dlsym(MRMediaRemoteSendCommand) failed: ...`).

### 5.4 Platform gating

`#[cfg(target_os = "macos")]` makes the entire module macOS-only. On other
build targets the function does not exist and the listener side has the
same gate.

### 5.5 Test strategy

- Constants (`MR_TOGGLE_PLAY_PAUSE`, `MEDIA_REMOTE_PATH`) are checked via
  inline unit tests.
- The actual `post_play_pause_key` invocation is verified manually because
  it depends on macOS Now Playing state and the private framework.
