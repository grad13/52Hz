// meta: updated=2026-03-07 14:55 checked=-
/**
 * settings-store - Presence & Tick Volume Unit Tests
 *
 * Tests for: frontend/lib/settings-store.ts (loadTickVolume, saveTickVolume, loadPresenceToast, savePresenceToast, loadPresencePosition, savePresencePosition, loadPresenceLevel, savePresenceLevel)
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

import {
  loadTickVolume,
  saveTickVolume,
  loadPresenceToast,
  savePresenceToast,
  loadPresencePosition,
  savePresencePosition,
  loadPresenceLevel,
  savePresenceLevel,
} from '@code/frontend/lib/settings-store';

// ---------------------------------------------------------------------------
// loadTickVolume
// ---------------------------------------------------------------------------
describe('loadTickVolume', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns 0.5 when store has boolean true (backward compat)', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'tick_sound') return Promise.resolve(true);
      return Promise.resolve(null);
    });

    const result = await loadTickVolume();
    expect(result).toBe(0.5);
  });

  it('returns 0 when store has boolean false (backward compat)', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'tick_sound') return Promise.resolve(false);
      return Promise.resolve(null);
    });

    const result = await loadTickVolume();
    expect(result).toBe(0);
  });

  it('returns 0 when store has null (not set)', async () => {
    mockStore.get.mockResolvedValue(null);

    const result = await loadTickVolume();
    expect(result).toBe(0);
  });

  it('returns the number when store has a number', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'tick_sound') return Promise.resolve(0.7);
      return Promise.resolve(null);
    });

    const result = await loadTickVolume();
    expect(result).toBe(0.7);
  });

  it('returns 0 on error', async () => {
    mockStore.get.mockRejectedValue(new Error('Store not available'));

    const result = await loadTickVolume();
    expect(result).toBe(0);
  });

  it('reads from key "tick_sound"', async () => {
    mockStore.get.mockResolvedValue(null);

    await loadTickVolume();

    expect(mockStore.get).toHaveBeenCalledWith('tick_sound');
  });
});

// ---------------------------------------------------------------------------
// saveTickVolume
// ---------------------------------------------------------------------------
describe('saveTickVolume', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('writes volume number to "tick_sound" key', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await saveTickVolume(0.8);

    expect(mockStore.set).toHaveBeenCalledWith('tick_sound', 0.8);
  });

  it('calls store.set exactly 1 time', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await saveTickVolume(0.5);

    expect(mockStore.set).toHaveBeenCalledTimes(1);
  });

  it('propagates errors from store.set to the caller', async () => {
    mockStore.set.mockRejectedValue(new Error('Write failed'));

    await expect(saveTickVolume(0.5)).rejects.toThrow('Write failed');
  });
});

// ---------------------------------------------------------------------------
// loadPresenceToast
// ---------------------------------------------------------------------------
describe('loadPresenceToast', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns true when presence_toast is true in store', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'presence_toast') return Promise.resolve(true);
      return Promise.resolve(null);
    });

    const result = await loadPresenceToast();
    expect(result).toBe(true);
  });

  it('returns false when presence_toast is false in store', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'presence_toast') return Promise.resolve(false);
      return Promise.resolve(null);
    });

    const result = await loadPresenceToast();
    expect(result).toBe(false);
  });

  it('returns true (default) when presence_toast is null', async () => {
    mockStore.get.mockResolvedValue(null);

    const result = await loadPresenceToast();
    expect(result).toBe(true);
  });

  it('returns true on error', async () => {
    mockStore.get.mockRejectedValue(new Error('Store not available'));

    const result = await loadPresenceToast();
    expect(result).toBe(true);
  });

  it('reads from key "presence_toast"', async () => {
    mockStore.get.mockResolvedValue(null);

    await loadPresenceToast();

    expect(mockStore.get).toHaveBeenCalledWith('presence_toast');
  });
});

// ---------------------------------------------------------------------------
// savePresenceToast
// ---------------------------------------------------------------------------
describe('savePresenceToast', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('writes true via store.set("presence_toast", true)', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePresenceToast(true);

    expect(mockStore.set).toHaveBeenCalledWith('presence_toast', true);
  });

  it('writes false via store.set("presence_toast", false)', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePresenceToast(false);

    expect(mockStore.set).toHaveBeenCalledWith('presence_toast', false);
  });

  it('calls store.set exactly 1 time', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePresenceToast(true);

    expect(mockStore.set).toHaveBeenCalledTimes(1);
  });

  it('propagates errors from store.set to the caller', async () => {
    mockStore.set.mockRejectedValue(new Error('Write failed'));

    await expect(savePresenceToast(true)).rejects.toThrow('Write failed');
  });
});

// ---------------------------------------------------------------------------
// loadPresencePosition
// ---------------------------------------------------------------------------
describe('loadPresencePosition', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns stored value when present', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'presence_position') return Promise.resolve('bottom-left');
      return Promise.resolve(null);
    });

    const result = await loadPresencePosition();
    expect(result).toBe('bottom-left');
  });

  it('returns "top-right" (default) when null', async () => {
    mockStore.get.mockResolvedValue(null);

    const result = await loadPresencePosition();
    expect(result).toBe('top-right');
  });

  it('returns "top-right" on error', async () => {
    mockStore.get.mockRejectedValue(new Error('Store not available'));

    const result = await loadPresencePosition();
    expect(result).toBe('top-right');
  });

  it('reads from key "presence_position"', async () => {
    mockStore.get.mockResolvedValue(null);

    await loadPresencePosition();

    expect(mockStore.get).toHaveBeenCalledWith('presence_position');
  });
});

// ---------------------------------------------------------------------------
// savePresencePosition
// ---------------------------------------------------------------------------
describe('savePresencePosition', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('writes position string to "presence_position" key', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePresencePosition('bottom-left');

    expect(mockStore.set).toHaveBeenCalledWith('presence_position', 'bottom-left');
  });

  it('calls store.set exactly 1 time', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePresencePosition('top-right');

    expect(mockStore.set).toHaveBeenCalledTimes(1);
  });

  it('propagates errors from store.set to the caller', async () => {
    mockStore.set.mockRejectedValue(new Error('Write failed'));

    await expect(savePresencePosition('top-right')).rejects.toThrow('Write failed');
  });
});

// ---------------------------------------------------------------------------
// loadPresenceLevel
// ---------------------------------------------------------------------------
describe('loadPresenceLevel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns stored value when present (new 3-value)', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'presence_level') return Promise.resolve('always-back');
      return Promise.resolve(null);
    });

    const result = await loadPresenceLevel();
    expect(result).toBe('always-back');
  });

  it('migrates legacy "front" to "always-front"', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'presence_level') return Promise.resolve('front');
      return Promise.resolve(null);
    });

    const result = await loadPresenceLevel();
    expect(result).toBe('always-front');
  });

  it('migrates legacy "back" to "always-back"', async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === 'presence_level') return Promise.resolve('back');
      return Promise.resolve(null);
    });

    const result = await loadPresenceLevel();
    expect(result).toBe('always-back');
  });

  it('returns "dynamic" (default) when null', async () => {
    mockStore.get.mockResolvedValue(null);

    const result = await loadPresenceLevel();
    expect(result).toBe('dynamic');
  });

  it('returns "dynamic" on error', async () => {
    mockStore.get.mockRejectedValue(new Error('Store not available'));

    const result = await loadPresenceLevel();
    expect(result).toBe('dynamic');
  });

  it('reads from key "presence_level"', async () => {
    mockStore.get.mockResolvedValue(null);

    await loadPresenceLevel();

    expect(mockStore.get).toHaveBeenCalledWith('presence_level');
  });
});

// ---------------------------------------------------------------------------
// savePresenceLevel
// ---------------------------------------------------------------------------
describe('savePresenceLevel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('writes level string to "presence_level" key', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePresenceLevel('always-back');

    expect(mockStore.set).toHaveBeenCalledWith('presence_level', 'always-back');
  });

  it('calls store.set exactly 1 time', async () => {
    mockStore.set.mockResolvedValue(undefined);

    await savePresenceLevel('dynamic');

    expect(mockStore.set).toHaveBeenCalledTimes(1);
  });

  it('propagates errors from store.set to the caller', async () => {
    mockStore.set.mockRejectedValue(new Error('Write failed'));

    await expect(savePresenceLevel('always-front')).rejects.toThrow('Write failed');
  });
});
