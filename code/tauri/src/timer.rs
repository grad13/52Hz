use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimerPhase {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSettings {
    pub focus_duration_secs: u64,
    pub short_break_duration_secs: u64,
    pub long_break_duration_secs: u64,
    pub short_breaks_before_long: u32,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            focus_duration_secs: 20 * 60,
            short_break_duration_secs: 20,
            long_break_duration_secs: 3 * 60,
            short_breaks_before_long: 3,
        }
    }
}

/// Result of a phase transition after a phase completes
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseEvent {
    BreakStart,
    BreakEnd,
    PhaseChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub phase: TimerPhase,
    pub paused: bool,
    pub elapsed_secs: u64,
    pub phase_duration_secs: u64,
    pub short_break_count: u32,
    pub settings: TimerSettings,
}

impl TimerState {
    pub fn new(settings: TimerSettings) -> Self {
        Self {
            phase: TimerPhase::Focus,
            paused: false,
            elapsed_secs: 0,
            phase_duration_secs: settings.focus_duration_secs,
            short_break_count: 0,
            settings,
        }
    }

    pub fn remaining_secs(&self) -> u64 {
        self.phase_duration_secs.saturating_sub(self.elapsed_secs)
    }

    pub fn remaining_display(&self) -> String {
        let remaining = self.remaining_secs();
        let mins = remaining / 60;
        let secs = remaining % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    pub fn tray_title(&self) -> String {
        let remaining = self.remaining_display();
        let phase_label = match self.phase {
            TimerPhase::Focus => "",
            TimerPhase::ShortBreak => " (休憩)",
            TimerPhase::LongBreak => " (長い休憩)",
        };
        format!("{}{}", remaining, phase_label)
    }

    /// Apply new settings. Updates phase_duration_secs for the current phase
    /// so changes take effect immediately (not just on next transition).
    /// If the user has already elapsed longer than the new duration,
    /// clamps elapsed to the new duration so remaining shows 00:00.
    pub fn apply_settings(&mut self, settings: TimerSettings) {
        self.settings = settings;
        self.phase_duration_secs = match self.phase {
            TimerPhase::Focus => self.settings.focus_duration_secs,
            TimerPhase::ShortBreak => self.settings.short_break_duration_secs,
            TimerPhase::LongBreak => self.settings.long_break_duration_secs,
        };
        if self.elapsed_secs > self.phase_duration_secs {
            self.elapsed_secs = self.phase_duration_secs;
        }
    }

    /// Skip the current break (mirrors the skip_break command logic).
    /// Returns events if a skip occurred, empty if not in a break phase.
    pub fn skip_break(&mut self) -> Vec<PhaseEvent> {
        if self.phase == TimerPhase::ShortBreak || self.phase == TimerPhase::LongBreak {
            self.phase = TimerPhase::Focus;
            self.elapsed_secs = 0;
            self.phase_duration_secs = self.settings.focus_duration_secs;
            vec![PhaseEvent::BreakEnd, PhaseEvent::PhaseChanged]
        } else {
            vec![]
        }
    }

    /// Step 1: Increment the timer by one second.
    /// Call this before emitting the timer-tick event so the frontend
    /// can observe remaining=0 before a transition occurs.
    pub fn advance(&mut self) {
        if !self.paused {
            self.elapsed_secs += 1;
        }
    }

    /// Step 2: Check if a phase transition should occur and perform it.
    /// Returns a list of events. Call this AFTER emitting the timer-tick
    /// so the frontend sees remaining=0 before the phase changes.
    pub fn try_transition(&mut self) -> Vec<PhaseEvent> {
        if self.paused || self.elapsed_secs < self.phase_duration_secs {
            return vec![];
        }

        // Phase complete — transition
        let old_phase = self.phase;

        let next_phase = match self.phase {
            TimerPhase::Focus => {
                self.short_break_count += 1;
                if self.short_break_count > self.settings.short_breaks_before_long {
                    self.short_break_count = 0;
                    TimerPhase::LongBreak
                } else {
                    TimerPhase::ShortBreak
                }
            }
            TimerPhase::ShortBreak | TimerPhase::LongBreak => TimerPhase::Focus,
        };

        self.phase = next_phase;
        self.elapsed_secs = 0;
        self.phase_duration_secs = match next_phase {
            TimerPhase::Focus => self.settings.focus_duration_secs,
            TimerPhase::ShortBreak => self.settings.short_break_duration_secs,
            TimerPhase::LongBreak => self.settings.long_break_duration_secs,
        };

        let mut events = vec![PhaseEvent::PhaseChanged];
        match next_phase {
            TimerPhase::ShortBreak | TimerPhase::LongBreak => {
                events.push(PhaseEvent::BreakStart);
            }
            TimerPhase::Focus => {
                if old_phase == TimerPhase::ShortBreak || old_phase == TimerPhase::LongBreak {
                    events.push(PhaseEvent::BreakEnd);
                }
            }
        }

        events
    }

    /// Convenience: advance + try_transition in one call.
    /// Used in tests and any context where the intermediate remaining=0
    /// state doesn't need to be observed.
    #[cfg(test)]
    pub fn tick(&mut self) -> Vec<PhaseEvent> {
        self.advance();
        self.try_transition()
    }
}

pub type SharedTimerState = Arc<Mutex<TimerState>>;

pub fn spawn_timer(app_handle: tauri::AppHandle, state: SharedTimerState) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        // tokio::time::interval fires immediately on the first tick.
        // Skip it so the first real tick happens after 1 second.
        interval.tick().await;
        loop {
            interval.tick().await;

            let mut s = state.lock().await;

            // Step 1: Advance the timer (increment elapsed).
            s.advance();

            // Step 2: Emit the current state BEFORE checking for transitions.
            // This ensures the frontend sees remaining=0 before the phase changes.
            let title = s.tray_title();
            let handle = app_handle.clone();
            let _ = app_handle.run_on_main_thread(move || {
                if let Some(tray) = handle.tray_by_id("main-tray") {
                    let _ = tray.set_title(Some(&title));
                }
            });
            let _ = app_handle.emit("timer-tick", s.clone());

            // Step 3: Check for phase transition.
            let events = s.try_transition();

            // Step 4: Emit phase events (and a second timer-tick with the new state).
            if !events.is_empty() {
                let _ = app_handle.emit("timer-tick", s.clone());
            }
            for event in &events {
                match event {
                    PhaseEvent::PhaseChanged => {
                        if cfg!(debug_assertions) {
                            eprintln!(
                                "[RestRun] phase-changed → {:?} (duration={}s)",
                                s.phase, s.phase_duration_secs
                            );
                        }
                        let _ = app_handle.emit("phase-changed", s.clone());
                    }
                    PhaseEvent::BreakStart => {
                        if cfg!(debug_assertions) {
                            eprintln!("[RestRun] break-start → {:?}", s.phase);
                        }
                        let _ = app_handle.emit("break-start", s.clone());
                    }
                    PhaseEvent::BreakEnd => {
                        if cfg!(debug_assertions) {
                            eprintln!("[RestRun] break-end → back to Focus");
                        }
                        let _ = app_handle.emit("break-end", ());
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
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
        assert_eq!(s.tray_title(), "00:01 (休憩)");
    }

    #[test]
    fn tray_title_long_break_shows_label() {
        let mut s = TimerState::new(test_settings());
        s.phase = TimerPhase::LongBreak;
        s.phase_duration_secs = 2;
        assert_eq!(s.tray_title(), "00:02 (長い休憩)");
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

    // ---------------------------------------------------------------
    // tick — phase transitions
    // ---------------------------------------------------------------

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

    // ---------------------------------------------------------------
    // Serialization round-trip (TimerState is sent to frontend)
    // ---------------------------------------------------------------

    #[test]
    fn timer_state_serialization_roundtrip() {
        let mut s = TimerState::new(test_settings());
        tick_n(&mut s, 5); // some state

        let json = serde_json::to_string(&s).expect("serialize");
        let deserialized: TimerState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.phase, s.phase);
        assert_eq!(deserialized.paused, s.paused);
        assert_eq!(deserialized.elapsed_secs, s.elapsed_secs);
        assert_eq!(deserialized.phase_duration_secs, s.phase_duration_secs);
        assert_eq!(deserialized.short_break_count, s.short_break_count);
    }

    #[test]
    fn timer_settings_serialization_roundtrip() {
        let settings = test_settings();
        let json = serde_json::to_string(&settings).expect("serialize");
        let deserialized: TimerSettings = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.focus_duration_secs, settings.focus_duration_secs);
        assert_eq!(
            deserialized.short_break_duration_secs,
            settings.short_break_duration_secs
        );
        assert_eq!(
            deserialized.long_break_duration_secs,
            settings.long_break_duration_secs
        );
        assert_eq!(
            deserialized.short_breaks_before_long,
            settings.short_breaks_before_long
        );
    }

    // ---------------------------------------------------------------
    // Countdown display: remaining_secs at every tick
    // ---------------------------------------------------------------

    #[test]
    fn remaining_secs_counts_down_correctly_every_tick() {
        let mut s = TimerState::new(test_settings()); // focus=3, short=1, long=2

        // Focus phase: 3 → 2 → 1 → [transition] (remaining=0 never observed via tick)
        assert_eq!(s.remaining_secs(), 3);
        s.tick();
        assert_eq!(s.remaining_secs(), 2);
        s.tick();
        assert_eq!(s.remaining_secs(), 1);
        s.tick(); // transition happens atomically
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.remaining_secs(), 1); // short break duration

        // ShortBreak: 1 → [transition]
        s.tick(); // transition
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.remaining_secs(), 3); // back to focus duration
    }

    #[test]
    fn tray_title_at_every_tick_in_cycle() {
        let mut s = TimerState::new(test_settings()); // focus=3, short=1, long=2

        // Focus: 00:03, 00:02, 00:01, then transition (no 00:00 via tick)
        assert_eq!(s.tray_title(), "00:03");
        s.tick();
        assert_eq!(s.tray_title(), "00:02");
        s.tick();
        assert_eq!(s.tray_title(), "00:01");
        s.tick(); // → ShortBreak (advance + transition happen atomically)
        assert_eq!(s.tray_title(), "00:01 (休憩)");
        s.tick(); // → Focus (ShortBreak(1) transitions after 1 tick)
        assert_eq!(s.tray_title(), "00:03");
    }

    // ---------------------------------------------------------------
    // Fast timer settings (matching RESTRUN_TEST_FAST_TIMER)
    // ---------------------------------------------------------------

    #[test]
    fn fast_timer_complete_cycle() {
        // Matches the env var settings: focus=5, short=3, long=5, cycles=2
        let settings = TimerSettings {
            focus_duration_secs: 5,
            short_break_duration_secs: 3,
            long_break_duration_secs: 5,
            short_breaks_before_long: 2,
        };
        let mut s = TimerState::new(settings);

        // First Focus (5 ticks) → ShortBreak
        tick_n(&mut s, 5);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 1);
        assert_eq!(s.phase_duration_secs, 3);

        // ShortBreak (3 ticks) → Focus
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Second Focus (5 ticks) → ShortBreak
        tick_n(&mut s, 5);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 2);

        // ShortBreak (3 ticks) → Focus
        tick_n(&mut s, 3);
        assert_eq!(s.phase, TimerPhase::Focus);

        // Third Focus (5 ticks) → LongBreak (count=3, 3>2)
        tick_n(&mut s, 5);
        assert_eq!(s.phase, TimerPhase::LongBreak);
        assert_eq!(s.short_break_count, 0);
        assert_eq!(s.phase_duration_secs, 5);

        // LongBreak (5 ticks) → Focus
        tick_n(&mut s, 5);
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.short_break_count, 0);

        // Cycle restarts: Focus(5) → ShortBreak (count=1)
        tick_n(&mut s, 5);
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert_eq!(s.short_break_count, 1);
    }

    #[test]
    fn fast_timer_cycle_total_ticks() {
        // One complete cycle: F(5)+S(3)+F(5)+S(3)+F(5)+L(5) = 26 ticks
        let settings = TimerSettings {
            focus_duration_secs: 5,
            short_break_duration_secs: 3,
            long_break_duration_secs: 5,
            short_breaks_before_long: 2,
        };
        let mut s = TimerState::new(settings);
        tick_n(&mut s, 26);
        assert_eq!(s.phase, TimerPhase::Focus);
        assert_eq!(s.short_break_count, 0);
        assert_eq!(s.elapsed_secs, 0);
    }

    // ---------------------------------------------------------------
    // Settings persistence: frontend store ↔ backend round trip
    // ---------------------------------------------------------------

    /// Verify the exact conversion the frontend does:
    /// focusMinutes * 60 → focus_duration_secs
    /// shortBreakSecs → short_break_duration_secs (no conversion)
    /// longBreakMinutes * 60 → long_break_duration_secs
    #[test]
    fn frontend_settings_conversion_roundtrip() {
        // Simulating what the frontend does in handleSaveSettings
        let focus_minutes: u64 = 25;
        let short_break_secs: u64 = 30;
        let long_break_minutes: u64 = 5;
        let short_breaks_before_long: u32 = 4;

        let settings = TimerSettings {
            focus_duration_secs: focus_minutes * 60,
            short_break_duration_secs: short_break_secs,
            long_break_duration_secs: long_break_minutes * 60,
            short_breaks_before_long,
        };

        assert_eq!(settings.focus_duration_secs, 1500);
        assert_eq!(settings.short_break_duration_secs, 30);
        assert_eq!(settings.long_break_duration_secs, 300);
        assert_eq!(settings.short_breaks_before_long, 4);

        // Verify we can apply and the timer works
        let s = TimerState::new(settings);
        assert_eq!(s.phase_duration_secs, 1500);
        assert_eq!(s.remaining_display(), "25:00");
    }

    /// Verify JSON deserialization from the store format matches expectations.
    /// The store saves: {"focus_minutes":1,"short_break_secs":3,...}
    /// The frontend reads those and constructs TimerSettings.
    #[test]
    fn store_json_to_timer_settings() {
        let store_json = r#"{"focus_minutes":1,"short_break_secs":3,"long_break_minutes":1,"short_breaks_before_long":2}"#;
        let store: serde_json::Value = serde_json::from_str(store_json).unwrap();

        let focus_minutes = store["focus_minutes"].as_u64().unwrap();
        let short_break_secs = store["short_break_secs"].as_u64().unwrap();
        let long_break_minutes = store["long_break_minutes"].as_u64().unwrap();
        let short_breaks_before_long = store["short_breaks_before_long"].as_u64().unwrap() as u32;

        let settings = TimerSettings {
            focus_duration_secs: focus_minutes * 60,
            short_break_duration_secs: short_break_secs,
            long_break_duration_secs: long_break_minutes * 60,
            short_breaks_before_long,
        };

        assert_eq!(settings.focus_duration_secs, 60);
        assert_eq!(settings.short_break_duration_secs, 3);
        assert_eq!(settings.long_break_duration_secs, 60);
        assert_eq!(settings.short_breaks_before_long, 2);
    }

    // ---------------------------------------------------------------
    // Default settings match between frontend and backend
    // ---------------------------------------------------------------

    /// The emitted remaining_secs at each tick simulates what the frontend
    /// receives via timer-tick. Verify the countdown is smooth and includes
    /// 0 before transitioning.
    #[test]
    fn countdown_reaches_zero_before_phase_transition() {
        let mut s = TimerState::new(test_settings()); // focus=3, short=1

        // Use advance() + try_transition() separately to observe remaining=0
        // before the transition occurs (as the real timer loop does).
        let mut focus_remaining: Vec<u64> = vec![];
        focus_remaining.push(s.remaining_secs()); // initial state: 3

        // Tick through Focus phase using advance() then try_transition()
        for _ in 0..3 {
            s.advance();
            focus_remaining.push(s.remaining_secs());
            let events = s.try_transition();
            if !events.is_empty() {
                // Transition happened
                break;
            }
        }

        // The user sees: 3, 2, 1, 0 then transition to break
        // (remaining=0 is observable between advance() and try_transition())
        assert_eq!(
            focus_remaining,
            vec![3, 2, 1, 0],
            "Countdown should show [3, 2, 1, 0] before transition, got {:?}",
            focus_remaining
        );
        assert_eq!(s.phase, TimerPhase::ShortBreak);
    }

    #[test]
    fn default_settings_match_frontend_defaults() {
        let defaults = TimerSettings::default();

        // Frontend defaults: focusMinutes=20, shortBreakSecs=20,
        // longBreakMinutes=3, shortBreaksBeforeLong=3
        let frontend_focus_minutes: u64 = 20;
        let frontend_short_break_secs: u64 = 20;
        let frontend_long_break_minutes: u64 = 3;
        let frontend_short_breaks_before_long: u32 = 3;

        assert_eq!(
            defaults.focus_duration_secs,
            frontend_focus_minutes * 60,
            "Focus duration mismatch between frontend and backend defaults"
        );
        assert_eq!(
            defaults.short_break_duration_secs,
            frontend_short_break_secs,
            "Short break duration mismatch between frontend and backend defaults"
        );
        assert_eq!(
            defaults.long_break_duration_secs,
            frontend_long_break_minutes * 60,
            "Long break duration mismatch between frontend and backend defaults"
        );
        assert_eq!(
            defaults.short_breaks_before_long,
            frontend_short_breaks_before_long,
            "Short breaks before long mismatch between frontend and backend defaults"
        );
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
}
