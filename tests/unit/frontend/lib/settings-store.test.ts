/**
 * settings-store - Unit Tests
 *
 * Tests for: frontend/lib/settings-store.ts
 * Spec: documents/spec/frontend/lib/settings-store.md
 * Runtime: JS-ESM
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockStore = {
  get: vi.fn(),
  set: vi.fn(),
};
vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn().mockResolvedValue(mockStore),
}));

import { toTimerSettings, toDisplaySettings, loadSettings, saveSettings } from '../../../../code/frontend/lib/settings-store';
import type { DisplaySettings } from '../../../../code/frontend/lib/settings-store';

// ---------------------------------------------------------------------------
// 3.1 Unit conversion: toTimerSettings (Decision Table)
// ---------------------------------------------------------------------------
describe('toTimerSettings', () => {
  // Decision Table:
  // | DisplaySettings field    | conversion | TimerSettings field          |
  // |--------------------------|------------|------------------------------|
  // | focusMinutes             | * 60       | focus_duration_secs          |
  // | shortBreakSecs           | as-is      | short_break_duration_secs    |
  // | longBreakMinutes         | * 60       | long_break_duration_secs     |
  // | shortBreaksBeforeLong    | as-is      | short_breaks_before_long     |

  it('converts focusMinutes to focus_duration_secs (* 60)', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakSecs: 20,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result.focus_duration_secs).toBe(25 * 60);
  });

  it('copies shortBreakSecs to short_break_duration_secs as-is', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakSecs: 30,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result.short_break_duration_secs).toBe(30);
  });

  it('converts longBreakMinutes to long_break_duration_secs (* 60)', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakSecs: 20,
      longBreakMinutes: 10,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result.long_break_duration_secs).toBe(10 * 60);
  });

  it('copies shortBreaksBeforeLong to short_breaks_before_long as-is', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakSecs: 20,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 4,
    };
    const result = toTimerSettings(d);
    expect(result.short_breaks_before_long).toBe(4);
  });

  it('converts all fields correctly for typical values', () => {
    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakSecs: 20,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };
    const result = toTimerSettings(d);
    expect(result).toEqual({
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    });
  });

  it('handles minimum boundary values (1 minute, 1 sec, 1 minute, 1 break)', () => {
    const d: DisplaySettings = {
      focusMinutes: 1,
      shortBreakSecs: 1,
      longBreakMinutes: 1,
      shortBreaksBeforeLong: 1,
    };
    const result = toTimerSettings(d);
    expect(result).toEqual({
      focus_duration_secs: 60,
      short_break_duration_secs: 1,
      long_break_duration_secs: 60,
      short_breaks_before_long: 1,
    });
  });

  it('handles large values', () => {
    const d: DisplaySettings = {
      focusMinutes: 120,
      shortBreakSecs: 300,
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
  // | short_break_duration_secs    | as-is      | shortBreakSecs           |
  // | long_break_duration_secs     | / 60       | longBreakMinutes         |
  // | short_breaks_before_long     | as-is      | shortBreaksBeforeLong    |

  it('converts focus_duration_secs to focusMinutes (/ 60)', () => {
    const s = {
      focus_duration_secs: 1500,
      short_break_duration_secs: 20,
      long_break_duration_secs: 300,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result.focusMinutes).toBe(25);
  });

  it('copies short_break_duration_secs to shortBreakSecs as-is', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 45,
      long_break_duration_secs: 300,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result.shortBreakSecs).toBe(45);
  });

  it('converts long_break_duration_secs to longBreakMinutes (/ 60)', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 600,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result.longBreakMinutes).toBe(10);
  });

  it('copies short_breaks_before_long to shortBreaksBeforeLong as-is', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 300,
      short_breaks_before_long: 5,
    };
    const result = toDisplaySettings(s);
    expect(result.shortBreaksBeforeLong).toBe(5);
  });

  it('converts all fields correctly for typical values', () => {
    const s = {
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    };
    const result = toDisplaySettings(s);
    expect(result).toEqual({
      focusMinutes: 20,
      shortBreakSecs: 20,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    });
  });

  it('handles minimum boundary values', () => {
    const s = {
      focus_duration_secs: 60,
      short_break_duration_secs: 1,
      long_break_duration_secs: 60,
      short_breaks_before_long: 1,
    };
    const result = toDisplaySettings(s);
    expect(result).toEqual({
      focusMinutes: 1,
      shortBreakSecs: 1,
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
      shortBreakSecs: 20,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });

  it('preserves typical Pomodoro values through round-trip', () => {
    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakSecs: 300,
      longBreakMinutes: 15,
      shortBreaksBeforeLong: 4,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });

  it('preserves minimum boundary values through round-trip', () => {
    const d: DisplaySettings = {
      focusMinutes: 1,
      shortBreakSecs: 1,
      longBreakMinutes: 1,
      shortBreaksBeforeLong: 1,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });

  it('preserves large values through round-trip', () => {
    const d: DisplaySettings = {
      focusMinutes: 120,
      shortBreakSecs: 600,
      longBreakMinutes: 60,
      shortBreaksBeforeLong: 10,
    };
    expect(toDisplaySettings(toTimerSettings(d))).toEqual(d);
  });
});

// ---------------------------------------------------------------------------
// 3.2 loadSettings
// ---------------------------------------------------------------------------
describe('loadSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns DisplaySettings when all keys exist in store', async () => {
    mockStore.get.mockImplementation((key: string) => {
      const values: Record<string, number> = {
        focus_minutes: 25,
        short_break_secs: 30,
        long_break_minutes: 5,
        short_breaks_before_long: 4,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakSecs: 30,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 4,
    });
  });

  it('returns null when all keys are null (first launch / no settings saved)', async () => {
    mockStore.get.mockResolvedValue(null);

    const result = await loadSettings();
    expect(result).toBeNull();
  });

  it('fills missing keys with default values when some keys exist', async () => {
    // Only focusMinutes is saved; others are null
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'focus_minutes') return Promise.resolve(30);
      return Promise.resolve(null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 30,
      shortBreakSecs: 20,       // default
      longBreakMinutes: 3,      // default
      shortBreaksBeforeLong: 3, // default
    });
  });

  it('fills default for shortBreakSecs when only it is missing', async () => {
    mockStore.get.mockImplementation((key: string) => {
      const values: Record<string, number | null> = {
        focus_minutes: 25,
        short_break_secs: null,
        long_break_minutes: 5,
        short_breaks_before_long: 4,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakSecs: 20,  // default
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 4,
    });
  });

  it('fills default for longBreakMinutes when only it is missing', async () => {
    mockStore.get.mockImplementation((key: string) => {
      const values: Record<string, number | null> = {
        focus_minutes: 25,
        short_break_secs: 30,
        long_break_minutes: null,
        short_breaks_before_long: 4,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakSecs: 30,
      longBreakMinutes: 3,  // default
      shortBreaksBeforeLong: 4,
    });
  });

  it('fills default for shortBreaksBeforeLong when only it is missing', async () => {
    mockStore.get.mockImplementation((key: string) => {
      const values: Record<string, number | null> = {
        focus_minutes: 25,
        short_break_secs: 30,
        long_break_minutes: 5,
        short_breaks_before_long: null,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakSecs: 30,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 3,  // default
    });
  });

  it('returns null when store.get throws an error', async () => {
    mockStore.get.mockRejectedValue(new Error('Store not available'));

    const result = await loadSettings();
    expect(result).toBeNull();
  });

  it('returns null when load() itself throws an error', async () => {
    const { load } = await import('@tauri-apps/plugin-store');
    (load as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Plugin not available'));

    const result = await loadSettings();
    expect(result).toBeNull();
  });

  it('calls load with "settings.json" and autoSave: true', async () => {
    mockStore.get.mockResolvedValue(null);
    const { load } = await import('@tauri-apps/plugin-store');

    await loadSettings();

    expect(load).toHaveBeenCalledWith(
      'settings.json',
      expect.objectContaining({ autoSave: true }),
    );
  });

  it('calls store.get for each of the 4 keys', async () => {
    mockStore.get.mockResolvedValue(null);

    await loadSettings();

    expect(mockStore.get).toHaveBeenCalledWith('focus_minutes');
    expect(mockStore.get).toHaveBeenCalledWith('short_break_secs');
    expect(mockStore.get).toHaveBeenCalledWith('long_break_minutes');
    expect(mockStore.get).toHaveBeenCalledWith('short_breaks_before_long');
  });
});

// ---------------------------------------------------------------------------
// 3.3 saveSettings
// ---------------------------------------------------------------------------
describe('saveSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('writes all 4 fields individually via store.set()', async () => {
    mockStore.set.mockResolvedValue(undefined);

    const d: DisplaySettings = {
      focusMinutes: 25,
      shortBreakSecs: 30,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 4,
    };

    await saveSettings(d);

    expect(mockStore.set).toHaveBeenCalledWith('focus_minutes', 25);
    expect(mockStore.set).toHaveBeenCalledWith('short_break_secs', 30);
    expect(mockStore.set).toHaveBeenCalledWith('long_break_minutes', 5);
    expect(mockStore.set).toHaveBeenCalledWith('short_breaks_before_long', 4);
  });

  it('calls store.set exactly 4 times', async () => {
    mockStore.set.mockResolvedValue(undefined);

    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakSecs: 20,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };

    await saveSettings(d);

    expect(mockStore.set).toHaveBeenCalledTimes(4);
  });

  it('calls load with "settings.json" and autoSave: true', async () => {
    mockStore.set.mockResolvedValue(undefined);
    const { load } = await import('@tauri-apps/plugin-store');

    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakSecs: 20,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };

    await saveSettings(d);

    expect(load).toHaveBeenCalledWith(
      'settings.json',
      expect.objectContaining({ autoSave: true }),
    );
  });

  it('propagates errors from store.set to the caller', async () => {
    mockStore.set.mockRejectedValue(new Error('Write failed'));

    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakSecs: 20,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };

    await expect(saveSettings(d)).rejects.toThrow('Write failed');
  });

  it('propagates errors from load() to the caller', async () => {
    const { load } = await import('@tauri-apps/plugin-store');
    (load as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Plugin error'));

    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakSecs: 20,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };

    await expect(saveSettings(d)).rejects.toThrow('Plugin error');
  });
});
