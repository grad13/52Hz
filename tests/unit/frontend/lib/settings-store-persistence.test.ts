/**
 * settings-store - Persistence Unit Tests
 *
 * Tests for: frontend/lib/settings-store.ts (loadSettings, saveSettings, loadPauseMediaOnBreak, savePauseMediaOnBreak, loadHideTrayIcon, saveHideTrayIcon)
 * Spec: _documents/spec/frontend/lib/settings-store.md
 * Runtime: JS-ESM
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockStore } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
    set: vi.fn(),
  },
}));
vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn().mockResolvedValue(mockStore),
}));

import { loadSettings, saveSettings, loadPauseMediaOnBreak, savePauseMediaOnBreak, loadHideTrayIcon, saveHideTrayIcon } from '@code/frontend/lib/settings-store';
import type { DisplaySettings } from '@code/frontend/lib/settings-store';

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
        short_break_minutes: 2,
        long_break_minutes: 5,
        short_breaks_before_long: 4,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakMinutes: 2,
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
      shortBreakMinutes: 1,     // default
      longBreakMinutes: 3,      // default
      shortBreaksBeforeLong: 3, // default
    });
  });

  it('fills default for shortBreakMinutes when only it is missing', async () => {
    mockStore.get.mockImplementation((key: string) => {
      const values: Record<string, number | null> = {
        focus_minutes: 25,
        short_break_minutes: null,
        long_break_minutes: 5,
        short_breaks_before_long: 4,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakMinutes: 1,  // default
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 4,
    });
  });

  it('fills default for longBreakMinutes when only it is missing', async () => {
    mockStore.get.mockImplementation((key: string) => {
      const values: Record<string, number | null> = {
        focus_minutes: 25,
        short_break_minutes: 2,
        long_break_minutes: null,
        short_breaks_before_long: 4,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakMinutes: 2,
      longBreakMinutes: 3,  // default
      shortBreaksBeforeLong: 4,
    });
  });

  it('fills default for shortBreaksBeforeLong when only it is missing', async () => {
    mockStore.get.mockImplementation((key: string) => {
      const values: Record<string, number | null> = {
        focus_minutes: 25,
        short_break_minutes: 2,
        long_break_minutes: 5,
        short_breaks_before_long: null,
      };
      return Promise.resolve(values[key] ?? null);
    });

    const result = await loadSettings();
    expect(result).toEqual({
      focusMinutes: 25,
      shortBreakMinutes: 2,
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
    expect(mockStore.get).toHaveBeenCalledWith('short_break_minutes');
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
      shortBreakMinutes: 2,
      longBreakMinutes: 5,
      shortBreaksBeforeLong: 4,
    };

    await saveSettings(d);

    expect(mockStore.set).toHaveBeenCalledWith('focus_minutes', 25);
    expect(mockStore.set).toHaveBeenCalledWith('short_break_minutes', 2);
    expect(mockStore.set).toHaveBeenCalledWith('long_break_minutes', 5);
    expect(mockStore.set).toHaveBeenCalledWith('short_breaks_before_long', 4);
  });

  it('calls store.set exactly 4 times', async () => {
    mockStore.set.mockResolvedValue(undefined);

    const d: DisplaySettings = {
      focusMinutes: 20,
      shortBreakMinutes: 1,
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
      shortBreakMinutes: 1,
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
      shortBreakMinutes: 1,
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
      shortBreakMinutes: 1,
      longBreakMinutes: 3,
      shortBreaksBeforeLong: 3,
    };

    await expect(saveSettings(d)).rejects.toThrow('Plugin error');
  });
});

// ---------------------------------------------------------------------------
// 3.4 loadPauseMediaOnBreak
// ---------------------------------------------------------------------------
describe('loadPauseMediaOnBreak', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns true when pause_media_on_break is true in store', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'pause_media_on_break') return Promise.resolve(true);
      return Promise.resolve(null);
    });

    const result = await loadPauseMediaOnBreak();
    expect(result).toBe(true);
  });

  it('returns false when pause_media_on_break is false in store', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'pause_media_on_break') return Promise.resolve(false);
      return Promise.resolve(null);
    });

    const result = await loadPauseMediaOnBreak();
    expect(result).toBe(false);
  });

  it('returns false when pause_media_on_break is null (not set)', async () => {
    mockStore.get.mockResolvedValue(null);

    const result = await loadPauseMediaOnBreak();
    expect(result).toBe(false);
  });

  it('returns false when store.get throws an error', async () => {
    mockStore.get.mockRejectedValue(new Error('Store not available'));

    const result = await loadPauseMediaOnBreak();
    expect(result).toBe(false);
  });

  it('returns false when load() itself throws an error', async () => {
    const { load } = await import('@tauri-apps/plugin-store');
    (load as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Plugin not available'));

    const result = await loadPauseMediaOnBreak();
    expect(result).toBe(false);
  });

  it('calls store.get with "pause_media_on_break"', async () => {
    mockStore.get.mockResolvedValue(null);

    await loadPauseMediaOnBreak();

    expect(mockStore.get).toHaveBeenCalledWith('pause_media_on_break');
  });

  it('calls load with "settings.json" and autoSave: true', async () => {
    mockStore.get.mockResolvedValue(null);
    const { load } = await import('@tauri-apps/plugin-store');

    await loadPauseMediaOnBreak();

    expect(load).toHaveBeenCalledWith(
      'settings.json',
      expect.objectContaining({ autoSave: true }),
    );
  });
});

// ---------------------------------------------------------------------------
// 3.5 savePauseMediaOnBreak
// ---------------------------------------------------------------------------
describe('savePauseMediaOnBreak', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('writes true via store.set("pause_media_on_break", true)', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePauseMediaOnBreak(true);

    expect(mockStore.set).toHaveBeenCalledWith('pause_media_on_break', true);
  });

  it('writes false via store.set("pause_media_on_break", false)', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePauseMediaOnBreak(false);

    expect(mockStore.set).toHaveBeenCalledWith('pause_media_on_break', false);
  });

  it('calls store.set exactly 1 time', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePauseMediaOnBreak(true);

    expect(mockStore.set).toHaveBeenCalledTimes(1);
  });

  it('calls load with "settings.json" and autoSave: true', async () => {
    mockStore.set.mockResolvedValue(undefined);
    const { load } = await import('@tauri-apps/plugin-store');

    await savePauseMediaOnBreak(true);

    expect(load).toHaveBeenCalledWith(
      'settings.json',
      expect.objectContaining({ autoSave: true }),
    );
  });

  it('propagates errors from store.set to the caller', async () => {
    mockStore.set.mockRejectedValue(new Error('Write failed'));

    await expect(savePauseMediaOnBreak(true)).rejects.toThrow('Write failed');
  });

  it('propagates errors from load() to the caller', async () => {
    const { load } = await import('@tauri-apps/plugin-store');
    (load as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Plugin error'));

    await expect(savePauseMediaOnBreak(true)).rejects.toThrow('Plugin error');
  });
});

// ---------------------------------------------------------------------------
// 3.6 loadHideTrayIcon
// ---------------------------------------------------------------------------
describe('loadHideTrayIcon', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns true when hide_tray_icon is true in store', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'hide_tray_icon') return Promise.resolve(true);
      return Promise.resolve(null);
    });

    const result = await loadHideTrayIcon();
    expect(result).toBe(true);
  });

  it('returns false when hide_tray_icon is false in store', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'hide_tray_icon') return Promise.resolve(false);
      return Promise.resolve(null);
    });

    const result = await loadHideTrayIcon();
    expect(result).toBe(false);
  });

  it('returns false when hide_tray_icon is null (not set)', async () => {
    mockStore.get.mockResolvedValue(null);

    const result = await loadHideTrayIcon();
    expect(result).toBe(false);
  });

  it('returns false when store.get throws an error', async () => {
    mockStore.get.mockRejectedValue(new Error('Store not available'));

    const result = await loadHideTrayIcon();
    expect(result).toBe(false);
  });

  it('returns false when load() itself throws an error', async () => {
    const { load } = await import('@tauri-apps/plugin-store');
    (load as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Plugin not available'));

    const result = await loadHideTrayIcon();
    expect(result).toBe(false);
  });

  it('calls store.get with "hide_tray_icon"', async () => {
    mockStore.get.mockResolvedValue(null);

    await loadHideTrayIcon();

    expect(mockStore.get).toHaveBeenCalledWith('hide_tray_icon');
  });
});

// ---------------------------------------------------------------------------
// 3.7 saveHideTrayIcon
// ---------------------------------------------------------------------------
describe('saveHideTrayIcon', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('writes true via store.set("hide_tray_icon", true)', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await saveHideTrayIcon(true);

    expect(mockStore.set).toHaveBeenCalledWith('hide_tray_icon', true);
  });

  it('writes false via store.set("hide_tray_icon", false)', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await saveHideTrayIcon(false);

    expect(mockStore.set).toHaveBeenCalledWith('hide_tray_icon', false);
  });

  it('calls store.set exactly 1 time', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await saveHideTrayIcon(true);

    expect(mockStore.set).toHaveBeenCalledTimes(1);
  });

  it('propagates errors from store.set to the caller', async () => {
    mockStore.set.mockRejectedValue(new Error('Write failed'));

    await expect(saveHideTrayIcon(true)).rejects.toThrow('Write failed');
  });

  it('propagates errors from load() to the caller', async () => {
    const { load } = await import('@tauri-apps/plugin-store');
    (load as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Plugin error'));

    await expect(saveHideTrayIcon(true)).rejects.toThrow('Plugin error');
  });
});
