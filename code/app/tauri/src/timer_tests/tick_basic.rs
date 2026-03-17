// meta: updated=2026-03-16 07:20 checked=-
    use super::*;

    // ---------------------------------------------------------------
    // tick — basic behavior
    // ---------------------------------------------------------------

    #[test]
    fn tick_paused_does_nothing() {
        let mut s = TimerState::new(test_settings());
        s.paused = true;
        let events = s.tick();
        assert!(events.is_empty());
        assert_eq!(s.elapsed_secs, 0);
    }

    #[test]
    fn tick_paused_multiple_times_stays_frozen() {
        let mut s = TimerState::new(test_settings());
        s.paused = true;
        for _ in 0..100 {
            assert!(s.tick().is_empty());
        }
        assert_eq!(s.elapsed_secs, 0);
        assert_eq!(s.phase, TimerPhase::Focus);
    }

    #[test]
    fn tick_increments_elapsed() {
        let mut s = TimerState::new(test_settings());
        let events = s.tick();
        assert!(events.is_empty());
        assert_eq!(s.elapsed_secs, 1);
        assert_eq!(s.phase, TimerPhase::Focus);
    }

    #[test]
    fn tick_no_transition_before_duration() {
        let mut s = TimerState::new(test_settings()); // focus = 3
        s.tick(); // 1
        s.tick(); // 2
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.elapsed_secs, 2);
    }

    // ---------------------------------------------------------------
    // tick — phase transitions
    // ---------------------------------------------------------------

    #[test]
    fn tick_transitions_exactly_at_duration() {
        let mut s = TimerState::new(test_settings()); // focus = 3
        s.tick(); // 1 (remaining=2)
        s.tick(); // 2 (remaining=1)
        assert_eq!(s.phase, TimerPhase::Focus);
        let events = s.tick(); // 3 → transition (elapsed reaches duration)
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.elapsed_secs, 0);
        assert!(events.contains(&PhaseEvent::PhaseChanged));
        assert!(events.contains(&PhaseEvent::BreakStart));
    }

    #[test]
    fn tick_does_not_double_transition() {
        // After a transition, the next tick should NOT immediately transition again.
        let mut s = TimerState::new(test_settings()); // focus=3, short=1
        tick_n(&mut s, 3); // Focus(3) → ShortBreak
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.elapsed_secs, 0);

        // Short break is 1 second. Tick 1: transition.
        let events = s.tick(); // transition
        assert_eq!(s.phase, TimerPhase::Focus);
        assert!(events.contains(&PhaseEvent::PhaseChanged));

        // Next tick should NOT transition, just increment.
        let events = s.tick();
        assert!(events.is_empty());
        assert_eq!(s.elapsed_secs, 1);
    }

    #[test]
    fn focus_to_short_break() {
        let mut s = TimerState::new(test_settings());
        let events = tick_n(&mut s, 3); // focus=3 ticks

        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.elapsed_secs, 0);
        assert_eq!(s.phase_duration_secs, 1);
        assert_eq!(s.short_break_count, 1);
        assert!(events.contains(&PhaseEvent::PhaseChanged));
        assert!(events.contains(&PhaseEvent::BreakStart));
        assert!(!events.contains(&PhaseEvent::BreakEnd));
    }

    #[test]
    fn short_break_to_focus() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::ShortBreak;
        s.phase_duration_secs = 1;
        s.short_break_count = 1;

        let events = s.tick(); // 1 tick → transition

        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.elapsed_secs, 0);
        assert_eq!(s.phase_duration_secs, 3);
        assert!(events.contains(&PhaseEvent::PhaseChanged));
        assert!(events.contains(&PhaseEvent::BreakEnd));
        assert!(!events.contains(&PhaseEvent::BreakStart));
    }

    #[test]
    fn long_break_triggers_after_n_short_breaks() {
        let mut s = TimerState::new(test_settings()); // short_breaks_before_long = 2

        // Cycle 1: Focus(3 ticks) → ShortBreak
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 1);

        // ShortBreak(1 tick) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Cycle 2: Focus(3 ticks) → ShortBreak (count=2, 2 > 2? No → still ShortBreak)
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 2);

        // ShortBreak(1 tick) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Cycle 3: Focus(3 ticks) → LongBreak (count=3, 3 > 2? Yes)
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0); // reset
        assert_eq!(s.phase_duration_secs, 2);
    }

    #[test]
    fn long_break_to_focus() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::LongBreak;
        s.phase_duration_secs = 2;

        s.tick(); // elapsed=1, remaining=1
        assert_eq!(s.phase, TimerPhase::LongBreak);

        let events = s.tick(); // elapsed=2 → transition
        assert_eq!(s.phase, TimerPhase::Focus);
        assert!(events.contains(&PhaseEvent::BreakEnd));
    }

    #[test]
    fn short_break_count_preserved_across_short_break_cycle() {
        // After Focus → ShortBreak → Focus, short_break_count should stay.
        let mut s = TimerState::new(test_settings());
        tick_n(&mut s, 3); // Focus(3) → ShortBreak (count=1)
        tick_n(&mut s, 1); // ShortBreak(1) → Focus
        assert_eq!(s.short_break_count, 1); // preserved
    }

    #[test]
    fn short_break_count_resets_after_long_break() {
        let mut s = TimerState::new(test_settings());
        // Focus(3) → Short(1) → Focus(3) → Short(1) → Focus(3) → Long
        tick_n(&mut s, 3 + 1 + 3 + 1 + 3);
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0);
    }

    // ---------------------------------------------------------------
    // tick — full cycle validation
    // ---------------------------------------------------------------

    #[test]
    fn full_cycle_phase_sequence() {
        // short_breaks_before_long = 2:
        // Focus(3) → Short(1) → Focus(3) → Short(1) → Focus(3) → Long(2) → Focus(3) → ...
        let mut s = TimerState::new(test_settings());
        let seq = collect_phase_sequence(&mut s, 40);

        assert_eq!(seq[0], (TimerPhase::Focus, 3));
        assert_eq!(seq[1], (TimerPhase::ShortBreak, 1));
        assert_eq!(seq[2], (TimerPhase::Focus, 3));
        assert_eq!(seq[3], (TimerPhase::ShortBreak, 1));
        assert_eq!(seq[4], (TimerPhase::Focus, 3));
        assert_eq!(seq[5], (TimerPhase::LongBreak, 2));
        assert_eq!(seq[6], (TimerPhase::Focus, 3));
        assert_eq!(seq[7], (TimerPhase::ShortBreak, 1));
    }

    #[test]
    fn full_cycle_event_sequence() {
        // Verify exact events for a complete cycle.
        // short_breaks_before_long = 2:
        // Focus(3) → Short(1) → Focus(3) → Short(1) → Focus(3) → Long(2) = 13 ticks
        let mut s = TimerState::new(test_settings());
        let all = tick_n_all_events(&mut s, 3 + 1 + 3 + 1 + 3 + 2);

        // Ticks 1-2: Focus counting down (no events)
        assert!(all[0].is_empty());
        assert!(all[1].is_empty());
        // Tick 3: Focus → ShortBreak
        assert!(all[2].contains(&PhaseEvent::BreakStart));
        // Tick 4: ShortBreak → Focus
        assert!(all[3].contains(&PhaseEvent::BreakEnd));
        // Ticks 5-6: Focus counting down (no events)
        assert!(all[4].is_empty());
        assert!(all[5].is_empty());
        // Tick 7: Focus → ShortBreak
        assert!(all[6].contains(&PhaseEvent::BreakStart));
        // Tick 8: ShortBreak → Focus
        assert!(all[7].contains(&PhaseEvent::BreakEnd));
        // Ticks 9-10: Focus counting down
        assert!(all[8].is_empty());
        assert!(all[9].is_empty());
        // Tick 11: Focus → LongBreak
        assert!(all[10].contains(&PhaseEvent::BreakStart));
        // Tick 12: LongBreak counting down
        assert!(all[11].is_empty());
        // Tick 13: LongBreak → Focus
        assert!(all[12].contains(&PhaseEvent::BreakEnd));
    }

    #[test]
    fn multiple_full_cycles_consistent() {
        let mut s = TimerState::new(test_settings());
        // One full cycle = 3+1+3+1+3+2 = 13 ticks (2 short breaks before long).
        // Run 5 full cycles (65 ticks) and verify we're back in the same state.
        for _ in 0..5 {
            tick_n(&mut s, 3 + 1 + 3 + 1 + 3 + 2);
        }
        // After 5 complete cycles, should be back at Focus with count=0
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.short_break_count, 0);
        assert_eq!(s.elapsed_secs, 0);
    }
