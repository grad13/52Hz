    use super::*;

    // ---------------------------------------------------------------
    // tick — boundary: short_breaks_before_long = 1
    // ---------------------------------------------------------------

    #[test]
    fn one_short_break_before_long_when_threshold_is_1() {
        let settings = TimerSettings {
            focus_duration_secs: 2,
            short_break_duration_secs: 1,
            long_break_duration_secs: 1,
            short_breaks_before_long: 1,
        };
        let mut s = TimerState::new(settings);

        // Focus(2 ticks) → ShortBreak (count=1, 1 > 1? No)
        tick_n(&mut s, 2);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 1);

        // ShortBreak(1 tick) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Focus(2 ticks) → LongBreak (count=2, 2 > 1? Yes)
        tick_n(&mut s, 2);
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0);

        // LongBreak(1 tick) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Repeats: Focus(2 ticks) → ShortBreak again
        tick_n(&mut s, 2);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
    }

    // ---------------------------------------------------------------
    // tick — short_breaks_before_long correctness
    // ---------------------------------------------------------------

    /// The number of actual short breaks must exactly equal short_breaks_before_long.
    /// This test catches off-by-one errors in the threshold comparison.
    #[test]
    fn short_break_count_equals_setting() {
        for n in 1u32..=5 {
            let settings = TimerSettings {
                focus_duration_secs: 2,
                short_break_duration_secs: 1,
                long_break_duration_secs: 1,
                short_breaks_before_long: n,
            };
            let mut s = TimerState::new(settings);
            let mut short_breaks = 0u32;

            // Run until we hit a LongBreak
            for _ in 0..1000 {
                let prev = s.phase;
                s.tick();
                if s.phase == TimerPhase::ShortBreak && prev != TimerPhase::ShortBreak {
                    short_breaks += 1;
                }
                if s.phase == TimerPhase::LongBreak {
                    break;
                }
            }

            assert_eq!(
                short_breaks, n,
                "short_breaks_before_long={} should produce exactly {} short breaks, got {}",
                n, n, short_breaks
            );
        }
    }

    /// With short_breaks_before_long = 0, every focus goes directly to LongBreak.
    #[test]
    fn zero_short_breaks_goes_directly_to_long() {
        let settings = TimerSettings {
            focus_duration_secs: 2,
            short_break_duration_secs: 1,
            long_break_duration_secs: 1,
            short_breaks_before_long: 0,
        };
        let mut s = TimerState::new(settings);

        // Focus(2 ticks) → LongBreak (count=1, 1 > 0? Yes)
        tick_n(&mut s, 2);
        assert_eq!(s.phase, TimerPhase::LongBreak);

        // LongBreak(1 tick) → Focus
        tick_n(&mut s, 1);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Focus(2 ticks) → LongBreak again (always)
        tick_n(&mut s, 2);
        assert_eq!(s.phase, TimerPhase::LongBreak);
    }

    // ---------------------------------------------------------------
    // tick — boundary: 1-second durations
    // ---------------------------------------------------------------

    #[test]
    fn one_second_everything() {
        let settings = TimerSettings {
            focus_duration_secs: 1,
            short_break_duration_secs: 1,
            long_break_duration_secs: 1,
            short_breaks_before_long: 2,
        };
        let mut s = TimerState::new(settings);

        // Each phase completes in 1 tick.
        // short_breaks_before_long=2: F→S→F→S→F→L cycle

        // Focus: tick 1 → transition to Short (count=1)
        let events = s.tick();
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert!(events.contains(&PhaseEvent::BreakStart));

        // Short: tick 2 → transition to Focus
        let events = s.tick();
        assert_eq!(s.phase, TimerPhase::Focus);
        assert!(events.contains(&PhaseEvent::BreakEnd));

        // Focus: tick 3 → Short (count=2, 2>2? No)
        let events = s.tick();
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert!(events.contains(&PhaseEvent::BreakStart));

        // Short: tick 4 → Focus
        let events = s.tick();
        assert_eq!(s.phase, TimerPhase::Focus);
        assert!(events.contains(&PhaseEvent::BreakEnd));

        // Focus: tick 5 → Long (count=3, 3>2? Yes)
        let events = s.tick();
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert!(events.contains(&PhaseEvent::BreakStart));

        // Long: tick 6 → Focus
        let events = s.tick();
        assert_eq!(s.phase, TimerPhase::Focus);
        assert!(events.contains(&PhaseEvent::BreakEnd));
    }
