// meta: updated=2026-03-07 06:55 checked=-
/**
 * settings-store - Conversion Unit Tests
 *
 * Tests for: frontend/lib/settings-store.ts (toTimerSettings, toDisplaySettings)
 * Spec: _documents/spec/frontend/lib/settings-store.md
 * Runtime: JS-ESM
 */

import { describe, it, expect } from 'vitest';

import { toTimerSettings, toDisplaySettings } from '@code/frontend/lib/settings-store';
import type { DisplaySettings } from '@code/frontend/lib/settings-store';

// ---------------------------------------------------------------------------
// 3.1 Unit conversion: toTimerSettings (Decision Table)
// ---------------------------------------------------------------------------
describe('toTimerSettings', () => {
  // Decision Table:
  // | DisplaySettings field    | conversion | TimerSettings field          |
  // |--------------------------|------------|------------------------------|
  // | focusMinutes             | * 60       | focus_duration_secs          |
  // | shortBreakMinutes        | * 60       | short_break_duration_secs    |
  // | longBreakMinutes         | * 60       | long_break_duration_secs     |
  // | shortBreaksBeforeLong    | as-is      | short_breaks_before_long     |

  it('converts focusMinutes to focus_duration_secs (* 60)', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakMinutes: 1,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result.focus_duration_secs).toBe(25 * 60);
  });

  it('converts shortBreakMinutes to short_break_duration_secs (* 60)', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakMinutes: 2,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result.short_break_duration_secs).toBe(120);
  });

  it('converts longBreakMinutes to long_break_duration_secs (* 60)', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakMinutes: 1,
      longBreakMinutes: 10,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result.long_break_duration_secs).toBe(10 * 60);
  });

  it('copies shortBreaksBeforeLong to short_breaks_before_long as-is', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakMinutes: 1,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 4,
    };
    const result = toTimerSettings(d);
    expect(result.short_breaks_before_long).toBe(4);
  });

  it('converts all fields correctly for typical values', () => {
    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakMinutes: 1,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result).toEqual({
      focus_duration_secs: 1200,
      short_break_duration_secs: 60,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    });
  });

  it('handles minimum boundary values (1 minute, 1 minute, 1 minute, 1 break)', () => {
    const d: DisplaySettings = {
      focusMinutes: 1,
      shortBreakMinutes: 1,
      longBreakMinutes: 1,
      shortBreaksBeforeLong: 1,
    };
    const result = toTimerSettings(d);
    expect(result).toEqual({
      focus_duration_secs: 60,
      short_break_duration_secs: 60,
      long_break_duration_secs: 60,
      short_breaks_before_long: 1,
    });
  });

  it('handles large values', () => {
    const d: DisplaySettings = {
      focusMinutes: 120,
      shortBreakMinutes: 5,
      longBreakMinutes: 60,
      shortBreaksBeforeLong: 10,
    };
    const result = toTimerSettings(d);
    expect(result).toEqual({
      focus_duration_secs: 7200,
      short_break_duration_secs: 300,
      long_break_duration_secs: 3600,
      short_breaks_before_long: 10,
    });
  });
});

// ---------------------------------------------------------------------------
// 3.1 Unit conversion: toDisplaySettings (Decision Table)
// ---------------------------------------------------------------------------
describe('toDisplaySettings', () => {
  // Reverse Decision Table:
  // | TimerSettings field          | conversion | DisplaySettings field    |
  // |------------------------------|------------|--------------------------|
  // | focus_duration_secs          | / 60       | focusMinutes             |
  // | short_break_duration_secs    | / 60       | shortBreakMinutes        |
  // | long_break_duration_secs     | / 60       | longBreakMinutes         |
  // | short_breaks_before_long     | as-is      | shortBreaksBeforeLong    |

  it('converts focus_duration_secs to focusMinutes (/ 60)', () => {
    const s = {
      focus_duration_secs: 1500,
      short_break_duration_secs: 60,
      long_break_duration_secs: 300,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result.focusMinutes).toBe(25);
  });

  it('converts short_break_duration_secs to shortBreakMinutes (/ 60)', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 120,
      long_break_duration_secs: 300,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result.shortBreakMinutes).toBe(2);
  });

  it('converts long_break_duration_secs to longBreakMinutes (/ 60)', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 60,
      long_break_duration_secs: 600,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result.longBreakMinutes).toBe(10);
  });

  it('copies short_breaks_before_long to shortBreaksBeforeLong as-is', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 60,
      long_break_duration_secs: 300,
      short_breaks_before_long: 5,
    };
    const result = toDisplaySettings(s);
    expect(result.shortBreaksBeforeLong).toBe(5);
  });

  it('converts all fields correctly for typical values', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 60,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result).toEqual({
      focusMinutes: 20,
      shortBreakMinutes: 1,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    });
  });

  it('handles minimum boundary values', () => {
    const s = {
      focus_duration_secs: 60,
      short_break_duration_secs: 60,
      long_break_duration_secs: 60,
      short_breaks_before_long: 1,
    };
    const result = toDisplaySettings(s);
    expect(result).toEqual({
      focusMinutes: 1,
      shortBreakMinutes: 1,
      longBreakMinutes: 1,
      shortBreaksBeforeLong: 1,
    });
  });
});

// ---------------------------------------------------------------------------
// 3.1 Round-trip invariant
// ---------------------------------------------------------------------------
describe('round-trip invariant: toDisplaySettings(toTimerSettings(d)) === d', () => {
  it('preserves default values through round-trip', () => {
    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakMinutes: 1,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });

  it('preserves typical Pomodoro values through round-trip', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakMinutes: 5,
      longBreakMinutes: 15,
      shortBreaksBeforeLong: 4,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });

  it('preserves minimum boundary values through round-trip', () => {
    const d: DisplaySettings = {
      focusMinutes: 1,
      shortBreakMinutes: 1,
      longBreakMinutes: 1,
      shortBreaksBeforeLong: 1,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });

  it('preserves large values through round-trip', () => {
    const d: DisplaySettings = {
      focusMinutes: 120,
      shortBreakMinutes: 10,
      longBreakMinutes: 60,
      shortBreaksBeforeLong: 10,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });
});
