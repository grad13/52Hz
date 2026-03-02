use std::ffi::c_void;

const NX_KEYTYPE_PLAY: i64 = 16;
const MEDIA_KEY_SUBTYPE: i16 = 8;

/// Build the data1 field for a media key NSSystemDefined event.
///
/// Layout: `(key_code << 16) | (flags << 8)`
/// - flags = 0xA for key down, 0xB for key up
pub fn build_media_key_data1(key_code: i64, key_down: bool) -> i64 {
    let flags: i64 = if key_down { 0xA } else { 0xB };
    (key_code << 16) | (flags << 8)
}

extern "C" {
    fn CGEventPost(tap: u32, event: *mut c_void);
}

/// Send a Play/Pause media key press (key down + key up).
///
/// Uses NSEvent.otherEvent(systemDefined) to create the event,
/// then posts it via CGEventPost to kCGHIDEventTap.
///
/// Requires Accessibility permission on macOS.
#[cfg(target_os = "macos")]
pub fn send_play_pause() {
    use objc2::msg_send;
    use objc2_app_kit::{NSEvent, NSEventModifierFlags, NSEventType};
    use objc2_foundation::NSPoint;

    for key_down in [true, false] {
        let data1 = build_media_key_data1(NX_KEYTYPE_PLAY, key_down);
        let modifier_flags = NSEventModifierFlags(if key_down { 0xA00 } else { 0xB00 });

        unsafe {
            let event = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
                NSEventType::SystemDefined,
                NSPoint::ZERO,
                modifier_flags,
                0.0,
                0,
                None,
                MEDIA_KEY_SUBTYPE,
                data1 as isize,
                -1isize,
            );

            if let Some(event) = event {
                let cg_event: *mut c_void = msg_send![&*event, CGEvent];
                if !cg_event.is_null() {
                    CGEventPost(0, cg_event); // kCGHIDEventTap = 0
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_media_key_data1_key_down() {
        let data1 = build_media_key_data1(NX_KEYTYPE_PLAY, true);
        // (16 << 16) | (0xA << 8) = 0x100A00
        assert_eq!(data1, 0x10_0A00);
    }

    #[test]
    fn test_build_media_key_data1_key_up() {
        let data1 = build_media_key_data1(NX_KEYTYPE_PLAY, false);
        // (16 << 16) | (0xB << 8) = 0x100B00
        assert_eq!(data1, 0x10_0B00);
    }

    #[test]
    fn test_build_media_key_data1_key_code_in_upper_bits() {
        let data1 = build_media_key_data1(NX_KEYTYPE_PLAY, true);
        assert_eq!((data1 >> 16) & 0xFF, NX_KEYTYPE_PLAY);
    }

    #[test]
    fn test_build_media_key_data1_flags_in_middle_bits() {
        let down = build_media_key_data1(NX_KEYTYPE_PLAY, true);
        let up = build_media_key_data1(NX_KEYTYPE_PLAY, false);
        assert_eq!((down >> 8) & 0xF, 0xA);
        assert_eq!((up >> 8) & 0xF, 0xB);
    }

    #[test]
    fn test_constants() {
        assert_eq!(NX_KEYTYPE_PLAY, 16);
        assert_eq!(MEDIA_KEY_SUBTYPE, 8);
    }
}
