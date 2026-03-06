    use super::*;

    // ---------------------------------------------------------------
    // apply_settings (update_settings command logic)
    // ---------------------------------------------------------------

    #[test]
    fn apply_settings_during_focus_updates_duration() {
        let mut s = TimerState::new(test_settings()); // focus = 3
        s.tick(); // elapsed = 1

        s.apply_settings(TimerSettings {
            focus_duration_secs: 10,
            ..test_settings()
        });

        assert_eq!(s.phase_duration_secs, 10);
        assert_eq!(s.remaining_secs(), 9); // 10 - 1
        s.tick(); // 2
        assert_eq!(s.phase, TimerPhase::Focus); // still in focus
    }

    #[test]
    fn apply_settings_clamps_elapsed_if_over_new_duration() {
        let mut s = TimerState::new(TimerSettings {
            focus_duration_secs: 100,
            ..test_settings()
        });
        tick_n(&mut s, 50); // elapsed = 50

        s.apply_settings(TimerSettings {
            focus_duration_secs: 30,
            ..test_settings()
        });

        // elapsed clamped to new duration (shows 00:00), NOT reset to 0
        assert_eq!(s.elapsed_secs, 30);
        assert_eq!(s.phase_duration_secs, 30);
        assert_eq!(s.remaining_secs(), 0);

        // Next tick transitions to break
        let events = s.tick();
        assert!(events.contains(&PhaseEvent::BreakStart));
    }

    #[test]
    fn apply_settings_keeps_elapsed_if_under_new_duration() {
        let mut s = TimerState::new(TimerSettings {
            focus_duration_secs: 100,
            ..test_settings()
        });
        tick_n(&mut s, 20); // elapsed = 20

        s.apply_settings(TimerSettings {
            focus_duration_secs: 50,
            ..test_settings()
        });

        assert_eq!(s.elapsed_secs, 20); // kept
        assert_eq!(s.phase_duration_secs, 50);
    }

    #[test]
    fn apply_settings_during_break_does_not_affect_current_phase() {
        let mut s = TimerState::new(test_settings());
        tick_n(&mut s, 3); // Focus(3) → ShortBreak (duration = 1)
        assert_eq!(s.phase, TimerPhase::ShortBreak);

        let old_duration = s.phase_duration_secs;
        s.apply_settings(TimerSettings {
            focus_duration_secs: 999,
            ..test_settings()
        });

        assert_eq!(s.phase_duration_secs, old_duration);
    }

    #[test]
    fn apply_settings_during_break_affects_next_focus() {
        let mut s = TimerState::new(test_settings()); // focus=3, short=1

        tick_n(&mut s, 3); // Focus(3) → ShortBreak
        assert_eq!(s.phase, TimerPhase::ShortBreak);

        // Change focus to 10 during break
        s.apply_settings(TimerSettings {
            focus_duration_secs: 10,
            ..test_settings()
        });

        // ShortBreak ends → Focus with new duration
        tick_n(&mut s, 1); // ShortBreak(1)
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.phase_duration_secs, 10);

        // Verify new Focus takes 10 ticks
        tick_n(&mut s, 9);
        assert_eq!(s.phase, TimerPhase::Focus);
        s.tick();
        assert_eq!(s.phase, TimerPhase::ShortBreak);
    }

    #[test]
    fn apply_settings_changes_break_duration_for_next_break() {
        let mut s = TimerState::new(test_settings()); // short=1

        // Change short break to 5 during Focus
        s.apply_settings(TimerSettings {
            short_break_duration_secs: 5,
            ..test_settings()
        });

        tick_n(&mut s, 3); // Focus(3) → ShortBreak
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.phase_duration_secs, 5); // New short break duration

        // Break lasts 5 ticks
        tick_n(&mut s, 4);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        s.tick();
        assert_eq!(s.phase, TimerPhase::Focus);
    }

    #[test]
    fn apply_settings_changes_threshold_mid_cycle() {
        let mut s = TimerState::new(test_settings()); // threshold=2

        // Focus(3) → ShortBreak (count=1)
        tick_n(&mut s, 3);
        assert_eq!(s.short_break_count, 1);

        // ShortBreak(1) → Focus
        tick_n(&mut s, 1);

        // Lower threshold to 1
        s.apply_settings(TimerSettings {
            short_breaks_before_long: 1,
            ..test_settings()
        });

        // Focus(3) → count=2, 2>1? Yes → LongBreak
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::LongBreak);
    }

    #[test]
    fn apply_settings_elapsed_equals_new_duration_boundary() {
        let mut s = TimerState::new(TimerSettings {
            focus_duration_secs: 100,
            ..test_settings()
        });
        tick_n(&mut s, 30); // elapsed = 30
        assert_eq!(s.elapsed_secs, 30);

        // Set focus to exactly 30 (= elapsed)
        s.apply_settings(TimerSettings {
            focus_duration_secs: 30,
            ..test_settings()
        });

        // Current behavior: 30 > 30 is false → elapsed NOT reset
        // remaining_secs = 0 → shows "00:00"
        assert_eq!(s.remaining_secs(), 0);

        // Next tick transitions immediately
        let events = s.tick();
        assert!(events.contains(&PhaseEvent::BreakStart));
    }

    // ---------------------------------------------------------------
    // apply_settings: elapsed exceeds new duration
    // ---------------------------------------------------------------

    /// When the user shortens focus time below the already-elapsed time,
    /// the timer should transition to a break (not restart from zero).
    /// Example: User has been focusing for 15 minutes, then changes
    /// focus to 10 minutes. They've already exceeded 10 minutes, so
    /// the timer should show 00:00 and transition to break on the next tick.
    /// A 20-minute focus should take exactly 1200 ticks (20*60 seconds),
    /// not 1201. The user expects "20 minutes" to mean 20 minutes.
    #[test]
    fn phase_duration_equals_actual_elapsed_seconds() {
        let settings = TimerSettings {
            focus_duration_secs: 5,
            short_break_duration_secs: 3,
            long_break_duration_secs: 5,
            short_breaks_before_long: 2,
        };
        let mut s = TimerState::new(settings);

        // Focus should take exactly 5 ticks (5 seconds), not 6.
        let mut ticks = 0;
        while s.phase == TimerPhase::Focus {
            s.tick();
            ticks += 1;
        }
        assert_eq!(
            ticks, 5,
            "Focus duration is 5s, so it should take exactly 5 ticks. Got {}.",
            ticks
        );
    }

    /// apply_settings during a break should also update the current break's
    /// phase_duration_secs — not just store the value for later.
    #[test]
    fn apply_settings_during_break_updates_current_break_duration() {
        let mut s = TimerState::new(test_settings()); // focus=3, short=1
        tick_n(&mut s, 3); // Focus(3) → ShortBreak (duration=1)
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.phase_duration_secs, 1);

        // User changes short break to 5 seconds while in ShortBreak
        s.apply_settings(TimerSettings {
            short_break_duration_secs: 5,
            ..test_settings()
        });

        // The current break should now use the new duration
        assert_eq!(
            s.phase_duration_secs, 5,
            "Changing short break duration during a short break should update the current break"
        );
    }

    #[test]
    fn apply_settings_shorter_focus_transitions_instead_of_restart() {
        let mut s = TimerState::new(TimerSettings {
            focus_duration_secs: 100,
            ..test_settings()
        });
        tick_n(&mut s, 50); // elapsed = 50

        // User shortens focus to 30 (less than elapsed 50)
        s.apply_settings(TimerSettings {
            focus_duration_secs: 30,
            ..test_settings()
        });

        // The user has already focused longer than the new duration.
        // Timer should show 00:00 (remaining=0) and transition on next tick.
        assert_eq!(
            s.remaining_secs(),
            0,
            "When elapsed ({}) exceeds new focus duration ({}), should show 00:00.\n\
             Instead, remaining={} — the timer restarted from the full new duration.",
            50,
            30,
            s.remaining_secs()
        );

        // Next tick should trigger break transition
        let events = s.tick();
        assert!(
            events.contains(&PhaseEvent::BreakStart),
            "Should transition to break on the tick after apply_settings.\n\
             The user exceeded the new focus time and should get a break, not a restart.\n\
             Events: {:?}, phase: {:?}",
            events,
            s.phase
        );
    }
