/**
 * timer.ts - Unit Tests
 *
 * Tests for: frontend/lib/timer.ts
 * Spec: documents/spec/frontend/lib/timer.md
 * Runtime: JS-ESM
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  remainingSecs,
  formatTime,
  getTimerState,
  pauseTimer,
  resumeTimer,
  togglePause,
  skipBreak,
  updateSettings,
  closeBreakOverlay,
  quitApp,
  onTimerTick,
  onPhaseChanged,
  onBreakStart,
  onBreakEnd,
} from '@code/frontend/lib/timer';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeState(overrides: Partial<{
  phase: 'Focus' | 'ShortBreak' | 'LongBreak';
  paused: boolean;
  elapsed_secs: number;
  phase_duration_secs: number;
  short_break_count: number;
  settings: {
    focus_duration_secs: number;
    short_break_duration_secs: number;
    long_break_duration_secs: number;
    short_breaks_before_long: number;
  };
}> = {}) {
  return {
    phase: 'Focus' as const,
    paused: false,
    elapsed_secs: 0,
    phase_duration_secs: 1500,
    short_break_count: 0,
    settings: {
      focus_duration_secs: 1500,
      short_break_duration_secs: 300,
      long_break_duration_secs: 900,
      short_breaks_before_long: 3,
    },
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// 1.2 Pure functions
// ---------------------------------------------------------------------------

describe('1.2 remainingSecs', () => {
  it('1.2-1: returns phase_duration_secs - elapsed_secs when elapsed < duration', () => {
    const state = makeState({ phase_duration_secs: 1500, elapsed_secs: 100 });
    expect(remainingSecs(state)).toBe(1400);
  });

  it('1.2-2: returns 0 when elapsed_secs equals phase_duration_secs', () => {
    const state = makeState({ phase_duration_secs: 300, elapsed_secs: 300 });
    expect(remainingSecs(state)).toBe(0);
  });

  it('1.2-3: returns 0 when elapsed_secs exceeds phase_duration_secs (safety clamp)', () => {
    const state = makeState({ phase_duration_secs: 300, elapsed_secs: 350 });
    expect(remainingSecs(state)).toBe(0);
  });

  it('1.2-4: returns full duration when elapsed_secs is 0', () => {
    const state = makeState({ phase_duration_secs: 1500, elapsed_secs: 0 });
    expect(remainingSecs(state)).toBe(1500);
  });

  it('1.2-5: returns 1 when 1 second remains', () => {
    const state = makeState({ phase_duration_secs: 1500, elapsed_secs: 1499 });
    expect(remainingSecs(state)).toBe(1);
  });
});

describe('1.2 formatTime', () => {
  it('1.2-6: 1200 => "20:00"', () => {
    expect(formatTime(1200)).toBe('20:00');
  });

  it('1.2-7: 65 => "01:05"', () => {
    expect(formatTime(65)).toBe('01:05');
  });

  it('1.2-8: 0 => "00:00"', () => {
    expect(formatTime(0)).toBe('00:00');
  });

  it('1.2-9: 5 => "00:05"', () => {
    expect(formatTime(5)).toBe('00:05');
  });

  it('1.2-10: 60 => "01:00"', () => {
    expect(formatTime(60)).toBe('01:00');
  });

  it('1.2-11: 3661 => "61:01" (60+ minutes displayed as-is)', () => {
    expect(formatTime(3661)).toBe('61:01');
  });
});

// ---------------------------------------------------------------------------
// 1.3 Tauri invoke wrappers
// ---------------------------------------------------------------------------

describe('1.3 IPC wrappers', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('1.3-1: getTimerState invokes "get_timer_state" with no arguments', async () => {
    const mockState = makeState();
    vi.mocked(invoke).mockResolvedValue(mockState);

    const result = await getTimerState();

    expect(invoke).toHaveBeenCalledWith('get_timer_state');
    expect(result).toEqual(mockState);
  });

  it('1.3-2: pauseTimer invokes "pause_timer" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await pauseTimer();

    expect(invoke).toHaveBeenCalledWith('pause_timer');
  });

  it('1.3-3: resumeTimer invokes "resume_timer" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await resumeTimer();

    expect(invoke).toHaveBeenCalledWith('resume_timer');
  });

  it('1.3-4: togglePause invokes "toggle_pause" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(true);

    const result = await togglePause();

    expect(invoke).toHaveBeenCalledWith('toggle_pause');
    expect(result).toBe(true);
  });

  it('1.3-5: skipBreak invokes "skip_break" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await skipBreak();

    expect(invoke).toHaveBeenCalledWith('skip_break');
  });

  it('1.3-6: updateSettings invokes "update_settings" with { settings } argument', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    const settings = {
      focus_duration_secs: 1800,
      short_break_duration_secs: 600,
      long_break_duration_secs: 1200,
      short_breaks_before_long: 4,
    };

    await updateSettings(settings);

    expect(invoke).toHaveBeenCalledWith('update_settings', { settings });
  });

  it('1.3-7: closeBreakOverlay invokes "close_break_overlay" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await closeBreakOverlay();

    expect(invoke).toHaveBeenCalledWith('close_break_overlay');
  });

  it('1.3-8: quitApp invokes "quit_app" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await quitApp();

    expect(invoke).toHaveBeenCalledWith('quit_app');
  });
});

// ---------------------------------------------------------------------------
// 1.4 Event listeners
// ---------------------------------------------------------------------------

describe('1.4 Event listeners', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('1.4-1: onTimerTick listens to "timer-tick" event', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(unlisten);

    const cb = vi.fn();
    await onTimerTick(cb);

    expect(listen).toHaveBeenCalledWith('timer-tick', expect.any(Function));
  });

  it('1.4-2: onTimerTick callback receives event.payload as TimerState', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      (handler as Function)({ payload: makeState({ elapsed_secs: 42 }) });
      return unlisten;
    });

    const cb = vi.fn();
    await onTimerTick(cb);

    expect(cb).toHaveBeenCalledWith(makeState({ elapsed_secs: 42 }));
  });

  it('1.4-3: onTimerTick returns unlisten function', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(unlisten);

    const cb = vi.fn();
    const result = await onTimerTick(cb);

    expect(result).toBe(unlisten);
  });

  it('1.4-4: onPhaseChanged listens to "phase-changed" event', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(unlisten);

    const cb = vi.fn();
    await onPhaseChanged(cb);

    expect(listen).toHaveBeenCalledWith('phase-changed', expect.any(Function));
  });

  it('1.4-5: onPhaseChanged callback receives event.payload as TimerState', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      (handler as Function)({ payload: makeState({ phase: 'ShortBreak' }) });
      return unlisten;
    });

    const cb = vi.fn();
    await onPhaseChanged(cb);

    expect(cb).toHaveBeenCalledWith(makeState({ phase: 'ShortBreak' }));
  });

  it('1.4-6: onBreakStart listens to "break-start" event', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(unlisten);

    const cb = vi.fn();
    await onBreakStart(cb);

    expect(listen).toHaveBeenCalledWith('break-start', expect.any(Function));
  });

  it('1.4-7: onBreakStart callback receives event.payload as TimerState', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      (handler as Function)({ payload: makeState({ phase: 'LongBreak' }) });
      return unlisten;
    });

    const cb = vi.fn();
    await onBreakStart(cb);

    expect(cb).toHaveBeenCalledWith(makeState({ phase: 'LongBreak' }));
  });

  it('1.4-8: onBreakEnd listens to "break-end" event', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(unlisten);

    const cb = vi.fn();
    await onBreakEnd(cb);

    expect(listen).toHaveBeenCalledWith('break-end', expect.any(Function));
  });

  it('1.4-9: onBreakEnd callback receives no payload (called with no arguments)', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      (handler as Function)({ payload: undefined });
      return unlisten;
    });

    const cb = vi.fn();
    await onBreakEnd(cb);

    expect(cb).toHaveBeenCalledWith();
  });

  it('1.4-10: onBreakEnd returns unlisten function', async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(unlisten);

    const cb = vi.fn();
    const result = await onBreakEnd(cb);

    expect(result).toBe(unlisten);
  });
});
