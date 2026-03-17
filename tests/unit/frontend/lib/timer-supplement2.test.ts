// meta: updated=2026-03-07 06:55 checked=-
/**
 * timer.ts - Supplement Tests (2)
 *
 * Tests for: frontend/lib/timer.ts
 * Spec: _documents/spec/frontend/lib/timer.md
 * Runtime: JS-ESM
 *
 * Supplement for: tests/unit/frontend/lib/timer-supplement.test.ts
 * Missing: resetTimer (1.3-13)
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { resetTimer } from '@code/frontend/lib/timer';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

// ---------------------------------------------------------------------------
// 1.3 Tauri invoke wrappers - supplement 2
// ---------------------------------------------------------------------------

describe('1.3 IPC wrappers - supplement 2', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('1.3-13: resetTimer invokes "reset_timer" with no arguments', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await resetTimer();

    expect(invoke).toHaveBeenCalledWith('reset_timer');
  });
});
