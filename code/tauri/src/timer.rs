use serde::{Deserialize, Serialize};

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
            short_break_duration_secs: 60,
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
    FocusDone,
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
        match self.phase {
            TimerPhase::Focus => {
                // Focus done: pause and emit FocusDone.
                // The actual Break transition happens via accept_break().
                self.short_break_count += 1;
                self.paused = true;
                vec![PhaseEvent::FocusDone]
            }
            TimerPhase::ShortBreak | TimerPhase::LongBreak => {
                let old_phase = self.phase;
                self.phase = TimerPhase::Focus;
                self.elapsed_secs = 0;
                self.phase_duration_secs = self.settings.focus_duration_secs;

                let mut events = vec![PhaseEvent::PhaseChanged];
                if old_phase == TimerPhase::ShortBreak || old_phase == TimerPhase::LongBreak {
                    events.push(PhaseEvent::BreakEnd);
                }
                events
            }
        }
    }

    /// Accept break after FocusDone. Transitions from Focus (paused) to
    /// ShortBreak or LongBreak based on short_break_count.
    /// Pre-condition: phase == Focus && paused == true.
    pub fn accept_break(&mut self) -> Vec<PhaseEvent> {
        if self.phase != TimerPhase::Focus || !self.paused {
            return vec![];
        }

        let next_phase = if self.short_break_count > self.settings.short_breaks_before_long {
            self.short_break_count = 0;
            TimerPhase::LongBreak
        } else {
            TimerPhase::ShortBreak
        };

        self.phase = next_phase;
        self.elapsed_secs = 0;
        self.phase_duration_secs = match next_phase {
            TimerPhase::ShortBreak => self.settings.short_break_duration_secs,
            TimerPhase::LongBreak => self.settings.long_break_duration_secs,
            TimerPhase::Focus => unreachable!(),
        };
        self.paused = false;

        vec![PhaseEvent::PhaseChanged, PhaseEvent::BreakStart]
    }

    /// Extend focus duration after FocusDone.
    /// Pre-condition: phase == Focus && paused == true.
    pub fn extend_focus(&mut self, secs: u64) {
        if self.phase != TimerPhase::Focus || !self.paused {
            return;
        }
        self.phase_duration_secs = self.phase_duration_secs.saturating_add(secs);
        self.paused = false;
    }

    /// Skip break from FocusDone and start next Focus immediately.
    /// Pre-condition: phase == Focus && paused == true.
    pub fn skip_break_from_focus(&mut self) -> Vec<PhaseEvent> {
        if self.phase != TimerPhase::Focus || !self.paused {
            return vec![];
        }
        self.elapsed_secs = 0;
        self.phase_duration_secs = self.settings.focus_duration_secs;
        self.paused = false;
        vec![PhaseEvent::PhaseChanged]
    }

    /// Convenience: advance + try_transition in one call.
    /// Auto-accepts FocusDone so existing cycle tests work unchanged.
    /// Use tick_raw() to observe FocusDone directly.
    #[cfg(test)]
    pub fn tick(&mut self) -> Vec<PhaseEvent> {
        self.advance();
        let events = self.try_transition();
        if events.contains(&PhaseEvent::FocusDone) {
            let mut all = events;
            all.extend(self.accept_break());
            return all;
        }
        events
    }

    /// Raw advance + try_transition without auto-accepting FocusDone.
    #[cfg(test)]
    pub fn tick_raw(&mut self) -> Vec<PhaseEvent> {
        self.advance();
        self.try_transition()
    }
}

#[cfg(test)]
#[path = "timer_tests.rs"]
mod tests;
