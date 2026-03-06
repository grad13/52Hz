    use super::*;

    // ---------------------------------------------------------------
    // Stress: many cycles
    // ---------------------------------------------------------------

    #[test]
    fn thousand_ticks_no_panic() {
        let mut s = TimerState::new(test_settings());
        let mut break_starts = 0usize;
        let mut break_ends = 0usize;

        for _ in 0..1000 {
            let events = s.tick();
            if events.contains(&PhaseEvent::BreakStart) {
                break_starts += 1;
            }
            if events.contains(&PhaseEvent::BreakEnd) {
                break_ends += 1;
            }
        }

        // 1 cycle = 3+1+3+1+3+2 = 13 ticks, 3 breaks per cycle.
        // 1000 ticks ≈ 76 cycles ≈ 228 breaks
        assert!(break_starts > 100, "Too few break starts: {}", break_starts);
        assert!(break_ends > 100, "Too few break ends: {}", break_ends);
        // Every break start should have a matching end (within 1 for the last).
        assert!(
            break_starts.abs_diff(break_ends) <= 1,
            "Mismatched: starts={}, ends={}",
            break_starts,
            break_ends
        );
    }

    #[test]
    fn one_second_durations_thousand_ticks() {
        let settings = TimerSettings {
            focus_duration_secs: 1,
            short_break_duration_secs: 1,
            long_break_duration_secs: 1,
            short_breaks_before_long: 2,
        };
        let mut s = TimerState::new(settings);
        let mut transitions = 0usize;

        for _ in 0..1000 {
            let events = s.tick();
            if !events.is_empty() {
                transitions += 1;
            }
        }

        // With 1-second durations, each phase takes exactly 1 tick.
        // So every tick is a transition: 1000 transitions in 1000 ticks.
        assert_eq!(transitions, 1000);
    }
