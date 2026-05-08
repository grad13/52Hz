// meta: updated=2026-05-08 checked=-
//
// Media control via macOS private MediaRemote.framework.
//
// `MRMediaRemoteSendCommand(TogglePlayPause, NULL)` toggles whichever app
// currently holds Now Playing — Music, Spotify, Podcasts, or any browser tab
// (incl. Firefox) that registered with the HTML5 media session API.
//
// This API is *private* (no public header) but stable enough that
// `nowplaying-cli` and similar tools rely on it. We resolve the symbol
// dynamically with dlopen/dlsym so the binary still links if Apple ever
// removes the framework — we'll just panic at call time.
//
// No special entitlement is required: this is media control, not input
// injection, so neither Accessibility nor Input Monitoring is requested.

#[cfg(target_os = "macos")]
use std::ffi::{c_char, c_void, CString};

#[cfg(target_os = "macos")]
const RTLD_NOW: i32 = 2;

/// `kMRTogglePlayPause` — see MediaRemote.h (recovered from class-dump).
#[cfg(target_os = "macos")]
const MR_TOGGLE_PLAY_PAUSE: u32 = 2;

#[cfg(target_os = "macos")]
const MEDIA_REMOTE_PATH: &str =
    "/System/Library/PrivateFrameworks/MediaRemote.framework/MediaRemote";

#[cfg(target_os = "macos")]
extern "C" {
    fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

#[cfg(target_os = "macos")]
fn last_dl_error() -> String {
    unsafe {
        let p = dlerror();
        if p.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }
}

/// Send the Now Playing toggle (play↔pause) to whichever app currently owns
/// Now Playing. Used at break-start and break-end as a pair, so the original
/// playback state is restored.
///
/// Panics if MediaRemote.framework can't be loaded or the symbol can't be
/// resolved — fail loudly rather than no-op.
#[cfg(target_os = "macos")]
pub fn post_play_pause_key() {
    let path = CString::new(MEDIA_REMOTE_PATH).expect("path has no NUL bytes");
    let handle = unsafe { dlopen(path.as_ptr(), RTLD_NOW) };
    if handle.is_null() {
        panic!(
            "dlopen({}) failed: {}",
            MEDIA_REMOTE_PATH,
            last_dl_error()
        );
    }

    let symbol = CString::new("MRMediaRemoteSendCommand").expect("symbol has no NUL bytes");
    let func_ptr = unsafe { dlsym(handle, symbol.as_ptr()) };
    if func_ptr.is_null() {
        panic!(
            "dlsym(MRMediaRemoteSendCommand) failed: {}",
            last_dl_error()
        );
    }

    type SendCommand = unsafe extern "C" fn(u32, *mut c_void) -> bool;
    let send_command: SendCommand = unsafe { std::mem::transmute(func_ptr) };

    let ok = unsafe { send_command(MR_TOGGLE_PLAY_PAUSE, std::ptr::null_mut()) };
    log::info!(
        "media: MRMediaRemoteSendCommand(TogglePlayPause) → {}",
        ok
    );
}
