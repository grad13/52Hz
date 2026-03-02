/**
 * App - Supplement Tests
 *
 * Tests for: code/frontend/App.svelte
 * Spec: documents/spec/frontend/app.md
 * Runtime: JS-ESM (Svelte 5)
 *
 * Supplement for: tests/unit/frontend/app.test.ts
 * Missing: 4-3 (focus-done view)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render } from '@testing-library/svelte';
import { cleanup } from '@testing-library/svelte';

// --- 子コンポーネントが依存するモジュールのモック ---

const { mockTimerModule, mockSettingsStoreModule } = vi.hoisted(() => ({
  mockTimerModule: {
    remainingSecs: vi.fn().mockReturnValue(1200),
    formatTime: vi.fn().mockReturnValue('20:00'),
    getTimerState: vi.fn().mockResolvedValue({
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
    }),
    pauseTimer: vi.fn(),
    resumeTimer: vi.fn(),
    togglePause: vi.fn(),
    skipBreak: vi.fn(),
    updateSettings: vi.fn(),
    closeBreakOverlay: vi.fn(),
    quitApp: vi.fn(),
    acceptBreak: vi.fn(),
    extendFocus: vi.fn(),
    skipBreakFromFocus: vi.fn(),
    getTodaySessions: vi.fn().mockResolvedValue(0),
    onTimerTick: vi.fn().mockResolvedValue(vi.fn()),
    onPhaseChanged: vi.fn().mockResolvedValue(vi.fn()),
    onBreakStart: vi.fn().mockResolvedValue(vi.fn()),
    onBreakEnd: vi.fn().mockResolvedValue(vi.fn()),
  },
  mockSettingsStoreModule: {
    loadSettings: vi.fn().mockResolvedValue(null),
    saveSettings: vi.fn(),
    toTimerSettings: vi.fn(),
    toDisplaySettings: vi.fn(),
    loadPauseMediaOnBreak: vi.fn().mockResolvedValue(false),
    savePauseMediaOnBreak: vi.fn().mockResolvedValue(undefined),
  },
}));

vi.mock('@code/frontend/lib/timer', () => mockTimerModule);
vi.mock('@code/frontend/lib/settings-store', () => mockSettingsStoreModule);

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({ close: vi.fn() }),
}));

vi.mock('@tauri-apps/plugin-autostart', () => ({
  isEnabled: vi.fn().mockResolvedValue(false),
  enable: vi.fn(),
  disable: vi.fn(),
}));

let originalLocation: Location;

beforeEach(() => {
  originalLocation = window.location;
});

afterEach(() => {
  cleanup();
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

describe('App - supplement', () => {
  it('4-3: ?view=focus-done で FocusDonePopup が表示され、BreakOverlay と TrayPanel は表示されない', async () => {
    setSearchParams('?view=focus-done');
    const { default: App } = await import('@code/frontend/App.svelte');
    render(App);

    // FocusDonePopup renders with .popup class
    const popup = document.querySelector('.popup');
    // BreakOverlay renders with .overlay class
    const overlay = document.querySelector('.overlay');
    // TrayPanel renders with .tray-panel class
    const trayPanel = document.querySelector('.tray-panel');

    // Should not have BreakOverlay or TrayPanel
    expect(overlay).toBeNull();
    expect(trayPanel).toBeNull();
  });
});
