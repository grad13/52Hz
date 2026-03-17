// meta: updated=2026-03-16 07:20 checked=-
    use super::*;

    // ---------------------------------------------------------------
    // pause / resume interactions with tick
    // ---------------------------------------------------------------

    #[test]
    fn pause_mid_phase_then_resume() {
        let mut s = TimerState::new(test_settings()); // focus = 3
        s.tick(); // elapsed = 1
        s.paused = true;
        s.tick(); // paused, no change
        s.tick();
        assert_eq!(s.elapsed_secs, 1); // frozen

        s.paused = false;
        s.tick(); // elapsed = 2
        assert_eq!(s.elapsed_secs, 2);
        assert_eq!(s.phase, TimerPhase::Focus);

        let events = s.tick(); // elapsed = 3 → transition
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert!(events.contains(&PhaseEvent::BreakStart));
    }

    #[test]
    fn pause_at_exact_boundary_does_not_transition() {
        let mut s = TimerState::new(test_settings()); // focus = 3
        s.tick(); // 1
        s.tick(); // 2
        s.paused = true;
        let events = s.tick(); // paused at elapsed=2, no transition
        assert!(events.is_empty());
        assert_eq!(s.elapsed_secs, 2);
        assert_eq!(s.phase, TimerPhase::Focus);
    }

    #[test]
    fn pause_during_break_preserves_break_state() {
        let mut s = TimerState::new(test_settings());
        tick_n(&mut s, 3); // Focus(3) → ShortBreak
        assert_eq!(s.phase, TimerPhase::ShortBreak);

        s.paused = true;
        for _ in 0..10 {
            s.tick();
        }
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.elapsed_secs, 0);
    }
