/**
 * TrayPanel - Supplement Tests (from spec-to-tests)
 *
 * Tests for: frontend/components/TrayPanel.svelte
 * Spec: _documents/spec/frontend/components/tray-panel.md
 * Runtime: JS-ESM (Svelte 5)
 *
 * Supplement for: tests/unit/frontend/components/tray-panel.test.ts
 * Missing: handleAutostartChange, quitApp, onDestroy cleanup, todaySessions
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import TrayPanel from '@code/frontend/components/TrayPanel.svelte';

afterEach(() => { cleanup(); });

// --- Tauri API mocks ---

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
  emit: vi.fn().mockResolvedValue(undefined),
  emitTo: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({
    close: vi.fn(),
    setSize: vi.fn().mockResolvedValue(undefined),
  }),
}));

// --- autostart plugin mock ---

const {
  mockIsEnabled,
  mockEnable,
  mockDisable,
} = vi.hoisted(() => ({
  mockIsEnabled: vi.fn(),
  mockEnable: vi.fn(),
  mockDisable: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-autostart', () => ({
  isEnabled: mockIsEnabled,
  enable: mockEnable,
  disable: mockDisable,
}));

// --- timer.ts mock ---

const {
  mockGetTimerState,
  mockTogglePause,
  mockUpdateSettings,
  mockQuitApp,
  mockOnTimerTick,
  mockOnPhaseChanged,
  mockRemainingSecs,
  mockFormatTime,
  mockGetTodaySessions,
} = vi.hoisted(() => ({
  mockGetTimerState: vi.fn(),
  mockTogglePause: vi.fn(),
  mockUpdateSettings: vi.fn(),
  mockQuitApp: vi.fn(),
  mockOnTimerTick: vi.fn(),
  mockOnPhaseChanged: vi.fn(),
  mockRemainingSecs: vi.fn(),
  mockFormatTime: vi.fn(),
  mockGetTodaySessions: vi.fn(),
}));

vi.mock('@code/frontend/lib/timer', () => ({
  remainingSecs: mockRemainingSecs,
  formatTime: mockFormatTime,
  getTimerState: mockGetTimerState,
  pauseTimer: vi.fn(),
  resumeTimer: vi.fn(),
  togglePause: mockTogglePause,
  skipBreak: vi.fn(),
  updateSettings: mockUpdateSettings,
  closeBreakOverlay: vi.fn(),
  quitApp: mockQuitApp,
  resetTimer: vi.fn(),
  onTimerTick: mockOnTimerTick,
  onPhaseChanged: mockOnPhaseChanged,
  onBreakStart: vi.fn().mockResolvedValue(vi.fn()),
  onBreakEnd: vi.fn().mockResolvedValue(vi.fn()),
  getTodaySessions: mockGetTodaySessions,
  acceptBreak: vi.fn(),
  extendFocus: vi.fn(),
  skipBreakFromFocus: vi.fn(),
  setTrayIconVisible: vi.fn().mockResolvedValue(undefined),
  listCassettes: vi.fn().mockResolvedValue([]),
  switchCassette: vi.fn().mockResolvedValue(undefined),
  openCassetteFolder: vi.fn().mockResolvedValue(undefined),
}));

// --- settings-store mock ---

const {
  mockLoadSettings,
  mockSaveSettings,
  mockToTimerSettings,
} = vi.hoisted(() => ({
  mockLoadSettings: vi.fn(),
  mockSaveSettings: vi.fn(),
  mockToTimerSettings: vi.fn(),
}));

vi.mock('@code/frontend/lib/settings-store', () => ({
  loadSettings: mockLoadSettings,
  saveSettings: mockSaveSettings,
  toTimerSettings: mockToTimerSettings,
  toDisplaySettings: vi.fn(),
  loadPauseMediaOnBreak: vi.fn().mockResolvedValue(false),
  savePauseMediaOnBreak: vi.fn().mockResolvedValue(undefined),
  loadHideTrayIcon: vi.fn().mockResolvedValue(false),
  saveHideTrayIcon: vi.fn().mockResolvedValue(undefined),
  loadTickVolume: vi.fn().mockResolvedValue(0),
  saveTickVolume: vi.fn().mockResolvedValue(undefined),
  loadPresenceToast: vi.fn().mockResolvedValue(true),
  savePresenceToast: vi.fn().mockResolvedValue(undefined),
  loadPresencePosition: vi.fn().mockResolvedValue('top-right'),
  savePresencePosition: vi.fn().mockResolvedValue(undefined),
  loadPresenceLevel: vi.fn().mockResolvedValue('front'),
  savePresenceLevel: vi.fn().mockResolvedValue(undefined),
  loadPresenceMaxToasts: vi.fn().mockResolvedValue(4),
  savePresenceMaxToasts: vi.fn().mockResolvedValue(undefined),
  loadPresenceShowIcon: vi.fn().mockResolvedValue(true),
  savePresenceShowIcon: vi.fn().mockResolvedValue(undefined),
  loadPresenceLikeIcon: vi.fn().mockResolvedValue('heart'),
  savePresenceLikeIcon: vi.fn().mockResolvedValue(undefined),
  loadLocale: vi.fn().mockResolvedValue(null),
  saveLocale: vi.fn().mockResolvedValue(undefined),
}));

// --- Helpers ---

function makeTimerState(overrides: Record<string, unknown> = {}) {
  return {
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
    ...overrides,
  };
}

// --- Tests ---

describe('TrayPanel - supplement (spec-to-tests)', () => {
  const mockUnlistenTick = vi.fn();
  const mockUnlistenPhaseChanged = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockRemainingSecs.mockReturnValue(1200);
    mockFormatTime.mockReturnValue('20:00');
    mockGetTimerState.mockResolvedValue(makeTimerState());
    mockLoadSettings.mockResolvedValue(null);
    mockTogglePause.mockResolvedValue(true);
    mockIsEnabled.mockResolvedValue(false);
    mockEnable.mockResolvedValue(undefined);
    mockDisable.mockResolvedValue(undefined);
    mockGetTodaySessions.mockResolvedValue(0);
    mockOnTimerTick.mockResolvedValue(mockUnlistenTick);
    mockOnPhaseChanged.mockResolvedValue(mockUnlistenPhaseChanged);
    mockToTimerSettings.mockReturnValue({
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    });
  });

  // =========================================================================
  // 5. handleAutostartChange (spec 7.3)
  // =========================================================================

  describe('handleAutostartChange', () => {
    it('5-1: enable() is called when enabling autostart', async () => {
      mockIsEnabled.mockResolvedValue(false);

      render(TrayPanel);

      await vi.waitFor(() => {
        expect(mockGetTimerState).toHaveBeenCalled();
      });

      const label = screen.getByText('Launch at login');
      const row = label.closest('.toggle-row');
      const toggle = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
      expect(toggle).toBeTruthy();
      await fireEvent.click(toggle!);

      await vi.waitFor(() => {
        expect(mockEnable).toHaveBeenCalledTimes(1);
      });
    });

    it('5-2: disable() is called when disabling autostart', async () => {
      mockIsEnabled.mockResolvedValue(true);

      render(TrayPanel);

      await vi.waitFor(() => {
        expect(mockGetTimerState).toHaveBeenCalled();
      });

      const label = screen.getByText('Launch at login');
      const row = label.closest('.toggle-row');
      const toggle = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
      expect(toggle).toBeTruthy();
      await fireEvent.click(toggle!);

      await vi.waitFor(() => {
        expect(mockDisable).toHaveBeenCalledTimes(1);
      });
    });
  });

  // =========================================================================
  // 6. quitApp (spec 7.2)
  // =========================================================================

  it('6-1: clicking "Quit" button calls quitApp', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockGetTimerState).toHaveBeenCalled();
    });

    const quitButton = screen.getByText('Quit');
    await fireEvent.click(quitButton);

    expect(mockQuitApp).toHaveBeenCalledTimes(1);
  });

  // =========================================================================
  // 7. onDestroy cleanup (spec 6.3)
  // =========================================================================

  it('7-1: timer-tick unlisten is called on unmount', async () => {
    const { unmount } = render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockOnTimerTick).toHaveBeenCalledTimes(1);
    });

    unmount();

    expect(mockUnlistenTick).toHaveBeenCalledTimes(1);
  });

  it('7-2: phase-changed unlisten is called on unmount', async () => {
    const { unmount } = render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockOnPhaseChanged).toHaveBeenCalledTimes(1);
    });

    unmount();

    expect(mockUnlistenPhaseChanged).toHaveBeenCalledTimes(1);
  });

  // =========================================================================
  // 8. todaySessions (spec R6 / 6.1)
  // =========================================================================

  it('8-1: getTodaySessions is called on mount', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockGetTodaySessions).toHaveBeenCalledTimes(1);
    });
  });

  it('8-2: session count is displayed as "N done"', async () => {
    mockGetTodaySessions.mockResolvedValue(3);

    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('3 done')).toBeTruthy();
    });
  });
});
