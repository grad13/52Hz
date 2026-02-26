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
#[path = "timer_tests.rs"]
mod tests;
