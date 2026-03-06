    use super::*;

    // ---------------------------------------------------------------
    // skip_break method
    // ---------------------------------------------------------------

    #[test]
    fn skip_short_break_returns_to_focus() {
        let mut s = TimerState::new(test_settings());
        tick_n(&mut s, 3); // Focus(3) → ShortBreak
        assert_eq!(s.phase, TimerPhase::ShortBreak);

        let events = s.skip_break();
        assert!(events.contains(&PhaseEvent::BreakEnd));
        assert!(events.contains(&PhaseEvent::PhaseChanged));
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.elapsed_secs, 0);
        assert_eq!(s.phase_duration_secs, 3);
    }

    #[test]
    fn skip_long_break_returns_to_focus() {
        let mut s = TimerState::new(test_settings());
        // Focus(3) → Short(1) → Focus(3) → Short(1) → Focus(3) → LongBreak
        tick_n(&mut s, 3 + 1 + 3 + 1 + 3);
        assert_eq!(s.phase, TimerPhase::LongBreak);

        let events = s.skip_break();
        assert!(events.contains(&PhaseEvent::BreakEnd));
        assert_eq!(s.phase, TimerPhase::Focus);
    }

    #[test]
    fn skip_during_focus_is_noop() {
        let mut s = TimerState::new(test_settings());
        s.tick(); // elapsed = 1

        let events = s.skip_break();
        assert!(events.is_empty());
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.elapsed_secs, 1);
    }

    #[test]
    fn skip_preserves_short_break_count() {
        let mut s = TimerState::new(test_settings());
        tick_n(&mut s, 3); // Focus(3) → ShortBreak (count=1)
        assert_eq!(s.short_break_count, 1);

        s.skip_break();
        assert_eq!(s.short_break_count, 1); // preserved
    }

    #[test]
    fn timer_continues_normally_after_skip() {
        let mut s = TimerState::new(test_settings());
        tick_n(&mut s, 3); // Focus(3) → ShortBreak
        s.skip_break();

        // Timer should work normally after skip
        tick_n(&mut s, 3); // Focus(3) → next break
        // short_break_count was 1 before skip, now becomes 2, 2 > 2? No → ShortBreak
        assert_eq!(s.phase, TimerPhase::ShortBreak);
    }

    #[test]
    fn skip_then_full_cycle_to_long_break() {
        let mut s = TimerState::new(test_settings()); // threshold=2

        // Focus(3) → ShortBreak (count=1), then skip
        tick_n(&mut s, 3);
        assert_eq!(s.short_break_count, 1);
        s.skip_break();

        // Focus(3) → ShortBreak (count=2)
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 2);

        // ShortBreak(1) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Focus(3) → LongBreak (count=3, 3>2? Yes)
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0);
    }

    #[test]
    fn skip_all_short_breaks_still_reaches_long_break() {
        let mut s = TimerState::new(test_settings()); // threshold=2

        // Skip every ShortBreak
        for i in 1..=2 {
            tick_n(&mut s, 3); // Focus(3) → ShortBreak
            assert_eq!(s.phase, TimerPhase::ShortBreak);
            assert_eq!(s.short_break_count, i);
            s.skip_break();
        }

        // Focus(3) → LongBreak (count=3, 3>2)
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::LongBreak);
    }

    #[test]
    fn skip_long_break_cycle_restarts_correctly() {
        let mut s = TimerState::new(test_settings()); // threshold=2

        // Full cycle to LongBreak
        tick_n(&mut s, 3 + 1 + 3 + 1 + 3);
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0);

        // Skip LongBreak
        s.skip_break();

        // Cycle restarts: Focus(3) → ShortBreak (count=1)
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 1);

        // Full cycle again
        tick_n(&mut s, 1); // ShortBreak(1) → Focus
        tick_n(&mut s, 3); // Focus(3) → ShortBreak (count=2)
        tick_n(&mut s, 1); // ShortBreak(1) → Focus
        tick_n(&mut s, 3); // Focus(3) → LongBreak (count=3, 3>2)
        assert_eq!(s.phase, TimerPhase::LongBreak);
    }

    // ---------------------------------------------------------------
    // skip_break_from_focus: FocusDone → next Focus
    // ---------------------------------------------------------------

    #[test]
    fn skip_break_from_focus_resets_to_focus() {
        let mut s = TimerState::new(test_settings()); // focus=3
        tick_n_raw(&mut s, 3); // FocusDone
        let events = s.skip_break_from_focus();
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.elapsed_secs, 0);
        assert_eq!(s.phase_duration_secs, 3); // focus_duration
        assert!(!s.paused);
        assert!(events.contains(&PhaseEvent::PhaseChanged));
        assert!(!events.contains(&PhaseEvent::BreakStart));
        assert!(!events.contains(&PhaseEvent::BreakEnd));
    }

    #[test]
    fn skip_break_from_focus_preserves_count() {
        let mut s = TimerState::new(test_settings());
        tick_n_raw(&mut s, 3); // FocusDone, count=1
        assert_eq!(s.short_break_count, 1);
        s.skip_break_from_focus();
        assert_eq!(s.short_break_count, 1); // unchanged
    }

    #[test]
    fn skip_break_from_focus_during_break_is_noop() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::ShortBreak;
        s.phase_duration_secs = 1;
        s.paused = true;
        let events = s.skip_break_from_focus();
        assert!(events.is_empty());
        assert_eq!(s.phase, TimerPhase::ShortBreak);
    }
