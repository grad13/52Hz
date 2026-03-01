/**
 * timer.ts - Supplement Tests
 *
 * Tests for: frontend/lib/timer.ts
 * Spec: documents/spec/frontend/lib/timer.md
 * Runtime: JS-ESM
 *
 * Supplement for: tests/unit/frontend/lib/timer.test.ts
 * Missing: getTodaySessions, acceptBreak, extendFocus, skipBreakFromFocus
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  getTodaySessions,
  acceptBreak,
  extendFocus,
  skipBreakFromFocus,
} from '@code/frontend/lib/timer';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

// ---------------------------------------------------------------------------
// 1.3 Tauri invoke wrappers - supplement
// ---------------------------------------------------------------------------

describe('1.3 IPC wrappers - supplement', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('1.3-9: getTodaySessions invokes "get_today_sessions" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(5);

    const result = await getTodaySessions();

    expect(invoke).toHaveBeenCalledWith('get_today_sessions');
    expect(result).toBe(5);
  });

  it('1.3-10: acceptBreak invokes "accept_break" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await acceptBreak();

    expect(invoke).toHaveBeenCalledWith('accept_break');
  });

  it('1.3-11: extendFocus invokes "extend_focus" with { secs } argument', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await extendFocus(60);

    expect(invoke).toHaveBeenCalledWith('extend_focus', { secs: 60 });
  });

  it('1.3-12: skipBreakFromFocus invokes "skip_break_from_focus" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await skipBreakFromFocus();

    expect(invoke).toHaveBeenCalledWith('skip_break_from_focus');
  });
});
