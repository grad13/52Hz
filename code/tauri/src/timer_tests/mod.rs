// meta: checked=2026-03-07
    use super::*;

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

    fn test_settings() -> TimerSettings {
        TimerSettings {
            focus_duration_secs: 3,
            short_break_duration_secs: 1,
            long_break_duration_secs: 2,
            short_breaks_before_long: 2,
        }
    }

    /// Tick n times and return events from the last tick.
    fn tick_n(state: &mut TimerState, n: usize) -> Vec<PhaseEvent> {
        let mut last_events = vec![];
        for _ in 0..n {
            last_events = state.tick();
        }
        last_events
    }

    /// Tick n times using tick_raw (no FocusDone auto-accept).
    fn tick_n_raw(state: &mut TimerState, n: usize) -> Vec<PhaseEvent> {
        let mut last_events = vec![];
        for _ in 0..n {
            last_events = state.tick_raw();
        }
        last_events
    }

    /// Tick n times and return all events collected from every tick.
    fn tick_n_all_events(state: &mut TimerState, n: usize) -> Vec<Vec<PhaseEvent>> {
        (0..n).map(|_| state.tick()).collect()
    }

    /// Collect the sequence of (phase, phase_duration) at each transition.
    fn collect_phase_sequence(state: &mut TimerState, ticks: usize) -> Vec<(TimerPhase, u64)> {
        let mut seq = vec![(state.phase, state.phase_duration_secs)];
        for _ in 0..ticks {
            let prev_phase = state.phase;
            state.tick();
            if state.phase != prev_phase {
                seq.push((state.phase, state.phase_duration_secs));
            }
        }
        seq
    }

    mod new_and_display;
    mod tick_basic;
    mod tick_boundary;
    mod pause_resume;
    mod apply_settings;
    mod skip_break;
    mod focus_done;
    mod events;
    mod stress;
    mod serialization;
