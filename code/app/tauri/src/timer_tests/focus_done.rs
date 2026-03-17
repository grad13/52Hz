// meta: updated=2026-03-16 07:20 checked=-
    use super::*;

    // ---------------------------------------------------------------
    // FocusDone: intermediate state after Focus completion
    // ---------------------------------------------------------------

    #[test]
    fn focus_done_returns_event_and_pauses() {
        let mut s = TimerState::new(test_settings()); // focus=3
        // Use tick_raw to observe FocusDone directly
        let events = tick_n_raw(&mut s, 3);
        assert!(events.contains(&PhaseEvent::FocusDone));
        assert_eq!(s.phase, TimerPhase::Focus); // stays Focus
        assert!(s.paused);
    }

    #[test]
    fn focus_done_increments_short_break_count() {
        let mut s = TimerState::new(test_settings());
        assert_eq!(s.short_break_count, 0);
        tick_n_raw(&mut s, 3); // FocusDone
        assert_eq!(s.short_break_count, 1);
    }

    #[test]
    fn focus_done_does_not_change_phase() {
        let mut s = TimerState::new(test_settings());
        tick_n_raw(&mut s, 3);
        // Phase remains Focus, not ShortBreak
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.elapsed_secs, 3); // elapsed preserved
    }

    // ---------------------------------------------------------------
    // accept_break: FocusDone → Break
    // ---------------------------------------------------------------

    #[test]
    fn accept_break_to_short_break() {
        let mut s = TimerState::new(test_settings()); // threshold=2
        tick_n_raw(&mut s, 3); // FocusDone (count=1)
        let events = s.accept_break();
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.elapsed_secs, 0);
        assert_eq!(s.phase_duration_secs, 1); // short_break_duration
        assert!(!s.paused);
        assert!(events.contains(&PhaseEvent::PhaseChanged));
        assert!(events.contains(&PhaseEvent::BreakStart));
    }

    #[test]
    fn accept_break_to_long_break() {
        let mut s = TimerState::new(test_settings()); // threshold=2
        // Cycle through to count > threshold
        // F(3)→FD→accept→S(1)→F(3)→FD→accept→S(1)→F(3)→FD (count=3, 3>2)
        tick_n(&mut s, 3 + 1 + 3 + 1); // through 2 short cycles
        tick_n_raw(&mut s, 3); // FocusDone (count=3)
        assert_eq!(s.short_break_count, 3);
        let events = s.accept_break();
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0); // reset
        assert_eq!(s.phase_duration_secs, 2); // long_break_duration
        assert!(events.contains(&PhaseEvent::BreakStart));
    }

    #[test]
    fn accept_break_unpauses() {
        let mut s = TimerState::new(test_settings());
        tick_n_raw(&mut s, 3);
        assert!(s.paused);
        s.accept_break();
        assert!(!s.paused);
    }

    #[test]
    fn accept_break_during_break_is_noop() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::ShortBreak;
        s.phase_duration_secs = 1;
        s.paused = true;
        let events = s.accept_break();
        assert!(events.is_empty());
        assert_eq!(s.phase, TimerPhase::ShortBreak);
    }

    #[test]
    fn accept_break_during_unpaused_focus_is_noop() {
        let mut s = TimerState::new(test_settings());
        s.tick(); // Focus, elapsed=1, not paused
        let events = s.accept_break();
        assert!(events.is_empty());
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.elapsed_secs, 1);
    }

    // ---------------------------------------------------------------
    // extend_focus: FocusDone → extend + resume
    // ---------------------------------------------------------------

    #[test]
    fn extend_focus_increases_duration() {
        let mut s = TimerState::new(test_settings()); // focus=3
        tick_n_raw(&mut s, 3); // FocusDone
        assert_eq!(s.phase_duration_secs, 3);
        s.extend_focus(60); // +1 minute
        assert_eq!(s.phase_duration_secs, 63);
    }

    #[test]
    fn extend_focus_unpauses() {
        let mut s = TimerState::new(test_settings());
        tick_n_raw(&mut s, 3);
        assert!(s.paused);
        s.extend_focus(60);
        assert!(!s.paused);
    }

    #[test]
    fn extend_focus_preserves_elapsed() {
        let mut s = TimerState::new(test_settings()); // focus=3
        tick_n_raw(&mut s, 3); // elapsed=3
        s.extend_focus(60);
        assert_eq!(s.elapsed_secs, 3); // preserved
        assert_eq!(s.remaining_secs(), 60); // 63 - 3
    }

    #[test]
    fn extend_focus_multiple_times() {
        let mut s = TimerState::new(test_settings()); // focus=3
        tick_n_raw(&mut s, 3); // FocusDone, elapsed=3, duration=3
        s.extend_focus(60); // duration=63, unpaused
        // Timer continues; advance to new FocusDone
        for _ in 0..60 {
            let events = s.tick_raw();
            if events.contains(&PhaseEvent::FocusDone) {
                // Second FocusDone
                assert_eq!(s.elapsed_secs, 63);
                s.extend_focus(30); // duration=93
                assert_eq!(s.phase_duration_secs, 93);
                assert_eq!(s.remaining_secs(), 30);
                return;
            }
        }
        panic!("Expected second FocusDone after extending");
    }

    #[test]
    fn extend_focus_during_break_is_noop() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::ShortBreak;
        s.phase_duration_secs = 1;
        s.paused = true;
        s.extend_focus(60);
        assert_eq!(s.phase_duration_secs, 1); // unchanged
        assert!(s.paused); // still paused
    }

    // ---------------------------------------------------------------
    // Full cycle with FocusDone
    // ---------------------------------------------------------------

    #[test]
    fn full_cycle_with_focus_done_accept() {
        // Verify a full cycle using raw tick + explicit accept_break
        let mut s = TimerState::new(test_settings()); // focus=3, short=1, long=2, threshold=2

        // Focus(3) → FocusDone → accept → ShortBreak
        let events = tick_n_raw(&mut s, 3);
        assert!(events.contains(&PhaseEvent::FocusDone));
        s.accept_break();
        assert_eq!(s.phase, TimerPhase::ShortBreak);

        // ShortBreak(1) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Focus(3) → FocusDone → accept → ShortBreak
        tick_n_raw(&mut s, 3);
        s.accept_break();
        assert_eq!(s.phase, TimerPhase::ShortBreak);

        // ShortBreak(1) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Focus(3) → FocusDone → accept → LongBreak (count=3 > 2)
        tick_n_raw(&mut s, 3);
        s.accept_break();
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0);

        // LongBreak(2) → Focus
        tick_n(&mut s, 2);
        assert_eq!(s.phase, TimerPhase::Focus);
    }

    #[test]
    fn full_cycle_with_focus_done_skip() {
        let mut s = TimerState::new(test_settings()); // threshold=2

        // Focus(3) → FocusDone → skip → Focus (count stays 1)
        tick_n_raw(&mut s, 3);
        assert_eq!(s.short_break_count, 1);
        s.skip_break_from_focus();
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.short_break_count, 1);

        // Focus(3) → FocusDone → accept → ShortBreak (count=2)
        tick_n_raw(&mut s, 3);
        assert_eq!(s.short_break_count, 2);
        s.accept_break();
        assert_eq!(s.phase, TimerPhase::ShortBreak);
    }
