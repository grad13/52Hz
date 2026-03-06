    use super::*;

    // ---------------------------------------------------------------
    // Event correctness
    // ---------------------------------------------------------------

    #[test]
    fn phase_changed_event_always_present_on_transition() {
        let mut s = TimerState::new(test_settings());
        // Every transition should include PhaseChanged
        for _ in 0..50 {
            let events = s.tick();
            if !events.is_empty() {
                assert!(
                    events.contains(&PhaseEvent::PhaseChanged),
                    "Transition without PhaseChanged: {:?}",
                    events
                );
            }
        }
    }

    #[test]
    fn break_start_only_on_break_entry() {
        let mut s = TimerState::new(test_settings());
        for _ in 0..50 {
            let prev_phase = s.phase;
            let events = s.tick();
            if events.contains(&PhaseEvent::BreakStart) {
                assert!(
                    s.phase == TimerPhase::ShortBreak || s.phase == TimerPhase::LongBreak,
                    "BreakStart emitted but phase is {:?}",
                    s.phase
                );
                assert_eq!(
                    prev_phase,
                    TimerPhase::Focus,
                    "BreakStart should only come from Focus"
                );
            }
        }
    }

    #[test]
    fn break_end_only_on_focus_entry_from_break() {
        let mut s = TimerState::new(test_settings());
        for _ in 0..50 {
            let prev_phase = s.phase;
            let events = s.tick();
            if events.contains(&PhaseEvent::BreakEnd) {
                assert_eq!(s.phase, TimerPhase::Focus, "BreakEnd but not in Focus");
                assert!(
                    prev_phase == TimerPhase::ShortBreak || prev_phase == TimerPhase::LongBreak,
                    "BreakEnd from non-break phase: {:?}",
                    prev_phase
                );
            }
        }
    }

    #[test]
    fn no_events_during_non_transition_ticks() {
        let mut s = TimerState::new(test_settings());
        for _ in 0..50 {
            let prev_phase = s.phase;
            let events = s.tick();
            if s.phase == prev_phase && s.elapsed_secs > 0 {
                assert!(
                    events.is_empty(),
                    "Got events {:?} during non-transition tick (phase={:?}, elapsed={})",
                    events,
                    s.phase,
                    s.elapsed_secs
                );
            }
        }
    }
