/**
 * App - Unit Tests
 *
 * Tests for: code/frontend/App.svelte
 * Spec: documents/spec/frontend/app.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { cleanup } from '@testing-library/svelte';
import type { TimerState } from '@code/frontend/lib/timer';

/**
 * App.svelte は window.location.search を参照して
 * BreakOverlay / TrayPanel を排他的にマウントする。
 * テストでは Object.defineProperty で search を差し替える。
 *
 * 子コンポーネント (BreakOverlay, TrayPanel) は onMount で
 * Tauri API を呼ぶため、依存モジュールをモックして
 * Unhandled Rejection を防止する。
 */

// --- 子コンポーネントが依存するモジュールのモック ---

const { mockTimerModule, mockSettingsStoreModule } = vi.hoisted(() => {
  const defaultState: TimerState = {
    phase: 'Focus',
    paused: false,
    elapsed_secs: 0,
    phase_duration_secs: 1200,
    short_break_count: 0,
    settings: {
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    },
  };
  return {
  mockTimerModule: {
    remainingSecs: vi.fn().mockReturnValue(1200),
    formatTime: vi.fn().mockReturnValue('20:00'),
    getTimerState: vi.fn().mockResolvedValue(defaultState),
    pauseTimer: vi.fn(),
    resumeTimer: vi.fn(),
    togglePause: vi.fn(),
    skipBreak: vi.fn(),
    updateSettings: vi.fn(),
    resetTimer: vi.fn(),
    closeBreakOverlay: vi.fn(),
    quitApp: vi.fn(),
    onTimerTick: vi.fn().mockResolvedValue(vi.fn()),
    onPhaseChanged: vi.fn().mockResolvedValue(vi.fn()),
    onBreakStart: vi.fn().mockResolvedValue(vi.fn()),
    onBreakEnd: vi.fn().mockResolvedValue(vi.fn()),
    getTodaySessions: vi.fn().mockResolvedValue(0),
    acceptBreak: vi.fn(),
    extendFocus: vi.fn(),
    skipBreakFromFocus: vi.fn(),
  },
  mockSettingsStoreModule: {
    loadSettings: vi.fn().mockResolvedValue(null),
    saveSettings: vi.fn(),
    toTimerSettings: vi.fn(),
    toDisplaySettings: vi.fn(),
    loadPauseMediaOnBreak: vi.fn().mockResolvedValue(false),
    savePauseMediaOnBreak: vi.fn().mockResolvedValue(undefined),
  },
  };
});

vi.mock('@code/frontend/lib/timer', () => mockTimerModule);
vi.mock('@code/frontend/lib/settings-store', () => mockSettingsStoreModule);

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({ close: vi.fn() }),
}));

vi.mock('@tauri-apps/plugin-autostart', () => ({
  isEnabled: vi.fn().mockResolvedValue(false),
  enable: vi.fn().mockResolvedValue(undefined),
  disable: vi.fn().mockResolvedValue(undefined),
}));

let originalLocation: Location;

beforeEach(() => {
  originalLocation = window.location;
});

afterEach(() => {
  cleanup();
  // location を元に戻す
  Object.defineProperty(window, 'location', {
    value: originalLocation,
    writable: true,
    configurable: true,
  });
});

function setSearchParams(search: string) {
  Object.defineProperty(window, 'location', {
    value: { ...originalLocation, search },
    writable: true,
    configurable: true,
  });
}

describe('App', () => {
  it('4-1: ?view=break で BreakOverlay コンポーネントが表示される', async () => {
    setSearchParams('?view=break');
    // 動的インポートで location.search 変更後にモジュールを読み込む
    const { default: App } = await import('@code/frontend/App.svelte');
    render(App);
    const breakOverlay = document.querySelector('[data-testid="break-overlay"]')
      ?? document.querySelector('.overlay');
    const trayPanel = document.querySelector('[data-testid="tray-panel"]')
      ?? document.querySelector('.tray-panel');
    expect(trayPanel).toBeNull();
  });

  it('4-2: パラメータなしで TrayPanel コンポーネントが表示される', async () => {
    setSearchParams('');
    const { default: App } = await import('@code/frontend/App.svelte');
    render(App);
    const breakOverlay = document.querySelector('[data-testid="break-overlay"]')
      ?? document.querySelector('.overlay');
    expect(breakOverlay).toBeNull();
  });
});
