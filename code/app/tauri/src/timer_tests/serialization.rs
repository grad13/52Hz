// meta: updated=2026-03-16 07:20 checked=-
    use super::*;

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
        assert_eq!(s.tray_title(), "00:01 (break)");
        s.tick(); // → Focus (ShortBreak(1) transitions after 1 tick)
        assert_eq!(s.tray_title(), "00:03");
    }

    // ---------------------------------------------------------------
    // Fast timer settings (matching FIFTYTWOHZ_TEST_FAST_TIMER)
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
        let store_json = r#"{"focus_minutes":1,"short_break_minutes":2,"long_break_minutes":1,"short_breaks_before_long":2}"#;
        let store: serde_json::Value = serde_json::from_str(store_json).unwrap();

        let focus_minutes = store["focus_minutes"].as_u64().unwrap();
        let short_break_minutes = store["short_break_minutes"].as_u64().unwrap();
        let long_break_minutes = store["long_break_minutes"].as_u64().unwrap();
        let short_breaks_before_long = store["short_breaks_before_long"].as_u64().unwrap() as u32;

        let settings = TimerSettings {
            focus_duration_secs: focus_minutes * 60,
            short_break_duration_secs: short_break_minutes * 60,
            long_break_duration_secs: long_break_minutes * 60,
            short_breaks_before_long,
        };

        assert_eq!(settings.focus_duration_secs, 60);
        assert_eq!(settings.short_break_duration_secs, 120);
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
                // FocusDone: timer pauses, remaining=0 is observable
                assert!(events.contains(&PhaseEvent::FocusDone));
                break;
            }
        }

        // The user sees: 3, 2, 1, 0 then FocusDone (paused)
        // (remaining=0 is observable between advance() and try_transition())
        assert_eq!(
            focus_remaining,
            vec![3, 2, 1, 0],
            "Countdown should show [3, 2, 1, 0] before FocusDone, got {:?}",
            focus_remaining
        );
        // Phase stays Focus (paused) until accept_break
        assert_eq!(s.phase, TimerPhase::Focus);
        assert!(s.paused);

        // After accept_break, transitions to break
        let events = s.accept_break();
        assert_eq!(s.phase, TimerPhase::ShortBreak);
        assert!(events.contains(&PhaseEvent::BreakStart));
    }

    #[test]
    fn default_settings_match_frontend_defaults() {
        let defaults = TimerSettings::default();

        // Frontend defaults: focusMinutes=20, shortBreakMinutes=1,
        // longBreakMinutes=3, shortBreaksBeforeLong=3
        let frontend_focus_minutes: u64 = 20;
        let frontend_short_break_minutes: u64 = 1;
        let frontend_long_break_minutes: u64 = 3;
        let frontend_short_breaks_before_long: u32 = 3;

        assert_eq!(
            defaults.focus_duration_secs,
            frontend_focus_minutes * 60,
            "Focus duration mismatch between frontend and backend defaults"
        );
        assert_eq!(
            defaults.short_break_duration_secs,
            frontend_short_break_minutes * 60,
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
