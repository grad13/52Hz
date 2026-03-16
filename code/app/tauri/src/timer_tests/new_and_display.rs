    use super::*;

    // ---------------------------------------------------------------
    // TimerState::new
    // ---------------------------------------------------------------

    #[test]
    fn new_state_has_correct_defaults() {
        let s = TimerState::new(TimerSettings::default());
        assert_eq!(s.phase, TimerPhase::Focus);
        assert!(!s.paused);
        assert_eq!(s.elapsed_secs, 0);
        assert_eq!(s.phase_duration_secs, 20 * 60);
        assert_eq!(s.short_break_count, 0);
    }

    #[test]
    fn new_state_uses_custom_settings() {
        let s = TimerState::new(test_settings());
        assert_eq!(s.phase_duration_secs, 3);
        assert_eq!(s.settings.short_break_duration_secs, 1);
        assert_eq!(s.settings.long_break_duration_secs, 2);
        assert_eq!(s.settings.short_breaks_before_long, 2);
    }

    // ---------------------------------------------------------------
    // remaining_secs / remaining_display
    // ---------------------------------------------------------------

    #[test]
    fn remaining_secs_counts_down() {
        let mut s = TimerState::new(test_settings());
        assert_eq!(s.remaining_secs(), 3);
        s.elapsed_secs = 1;
        assert_eq!(s.remaining_secs(), 2);
        s.elapsed_secs = 3;
        assert_eq!(s.remaining_secs(), 0);
    }

    #[test]
    fn remaining_secs_saturates_at_zero() {
        let mut s = TimerState::new(test_settings());
        s.elapsed_secs = 100;
        assert_eq!(s.remaining_secs(), 0);
    }

    #[test]
    fn remaining_display_format() {
        let mut s = TimerState::new(TimerSettings::default());
        assert_eq!(s.remaining_display(), "20:00");
        s.elapsed_secs = 60;
        assert_eq!(s.remaining_display(), "19:00");
        s.elapsed_secs = 20 * 60 - 5;
        assert_eq!(s.remaining_display(), "00:05");
    }

    #[test]
    fn remaining_display_at_zero() {
        let mut s = TimerState::new(test_settings());
        s.elapsed_secs = s.phase_duration_secs;
        assert_eq!(s.remaining_display(), "00:00");
    }

    #[test]
    fn remaining_display_exactly_one_second() {
        let mut s = TimerState::new(test_settings());
        s.elapsed_secs = s.phase_duration_secs - 1;
        assert_eq!(s.remaining_display(), "00:01");
    }

    #[test]
    fn remaining_display_on_minute_boundary() {
        let mut s = TimerState::new(TimerSettings {
            focus_duration_secs: 120,
            ..test_settings()
        });
        s.elapsed_secs = 60;
        assert_eq!(s.remaining_display(), "01:00");
    }

    // ---------------------------------------------------------------
    // tray_title
    // ---------------------------------------------------------------

    #[test]
    fn tray_title_focus_shows_time_only() {
        let s = TimerState::new(test_settings());
        assert_eq!(s.tray_title(), "00:03");
    }

    #[test]
    fn tray_title_short_break_shows_label() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::ShortBreak;
        s.phase_duration_secs = 1;
        assert_eq!(s.tray_title(), "00:01 (break)");
    }

    #[test]
    fn tray_title_long_break_shows_label() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::LongBreak;
        s.phase_duration_secs = 2;
        assert_eq!(s.tray_title(), "00:02 (long break)");
    }

    #[test]
    fn tray_title_updates_as_time_elapses() {
        let mut s = TimerState::new(TimerSettings {
            focus_duration_secs: 65,
            ..test_settings()
        });
        assert_eq!(s.tray_title(), "01:05");
        s.elapsed_secs = 5;
        assert_eq!(s.tray_title(), "01:00");
        s.elapsed_secs = 64;
        assert_eq!(s.tray_title(), "00:01");
    }
