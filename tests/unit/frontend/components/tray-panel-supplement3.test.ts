/**
 * TrayPanel - Supplement Tests 3 (from spec-to-tests)
 *
 * Tests for: frontend/components/TrayPanel.svelte
 * Spec: _documents/spec/frontend/components/tray-panel.md
 * Runtime: JS-ESM (Svelte 5)
 *
 * Supplement for: tests/unit/frontend/components/tray-panel.test.ts
 * Missing: loadPauseMediaOnBreak mount call, handleAutostartChange error handling,
 *          loadSavedSettings null branch, handlePauseMediaChange end-to-end
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
}));

// --- settings-store mock ---

const {
  mockLoadSettings,
  mockSaveSettings,
  mockToTimerSettings,
  mockLoadPauseMediaOnBreak,
  mockSavePauseMediaOnBreak,
} = vi.hoisted(() => ({
  mockLoadSettings: vi.fn(),
  mockSaveSettings: vi.fn(),
  mockToTimerSettings: vi.fn(),
  mockLoadPauseMediaOnBreak: vi.fn(),
  mockSavePauseMediaOnBreak: vi.fn(),
}));

vi.mock('@code/frontend/lib/settings-store', () => ({
  loadSettings: mockLoadSettings,
  saveSettings: mockSaveSettings,
  toTimerSettings: mockToTimerSettings,
  toDisplaySettings: vi.fn(),
  loadPauseMediaOnBreak: mockLoadPauseMediaOnBreak,
  savePauseMediaOnBreak: mockSavePauseMediaOnBreak,
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

describe('TrayPanel - supplement 3 (spec-to-tests)', () => {
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
    mockOnTimerTick.mockResolvedValue(vi.fn());
    mockOnPhaseChanged.mockResolvedValue(vi.fn());
    mockLoadPauseMediaOnBreak.mockResolvedValue(false);
    mockSavePauseMediaOnBreak.mockResolvedValue(undefined);
    mockToTimerSettings.mockReturnValue({
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    });
  });

  // =========================================================================
  // 9. loadPauseMediaOnBreak mount call (spec 6.1 step 3 / R7)
  // =========================================================================

  describe('loadPauseMediaOnBreak on mount', () => {
    it('9-1: loadPauseMediaOnBreak is called on mount', async () => {
      render(TrayPanel);

      await vi.waitFor(() => {
        expect(mockLoadPauseMediaOnBreak).toHaveBeenCalledTimes(1);
      });
    });

    it('9-2: toggle is checked when loadPauseMediaOnBreak returns true', async () => {
      mockLoadPauseMediaOnBreak.mockResolvedValue(true);

      render(TrayPanel);

      await vi.waitFor(() => {
        const label = screen.getByText('Auto-pause');
        const row = label.closest('.toggle-row');
        const toggle = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
        expect(toggle).toBeTruthy();
        expect(toggle.checked).toBe(true);
      });
    });

    it('9-3: toggle is unchecked when loadPauseMediaOnBreak returns false', async () => {
      mockLoadPauseMediaOnBreak.mockResolvedValue(false);

      render(TrayPanel);

      await vi.waitFor(() => {
        const label = screen.getByText('Auto-pause');
        const row = label.closest('.toggle-row');
        const toggle = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
        expect(toggle).toBeTruthy();
        expect(toggle.checked).toBe(false);
      });
    });
  });

  // =========================================================================
  // 10. handleAutostartChange error handling (spec 7.3 step 3)
  // =========================================================================

  describe('handleAutostartChange error handling', () => {
    it('10-1: isEnabled() re-fetches state even when enable() fails', async () => {
      mockIsEnabled.mockResolvedValue(false);
      mockEnable.mockRejectedValue(new Error('autostart error'));
      // After error, isEnabled is called again to get real state
      mockIsEnabled.mockResolvedValueOnce(false) // initial mount
        .mockResolvedValueOnce(false); // fallback after error

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
        // isEnabled should be called: once on mount + once as fallback
        expect(mockIsEnabled.mock.calls.length).toBeGreaterThanOrEqual(2);
      });
    });
  });

  // =========================================================================
  // 11. loadSavedSettings null branch (spec 6.1 step 1c)
  // =========================================================================

  describe('loadSavedSettings null branch', () => {
    it('11-1: updateSettings is not called when loadSettings returns null (defaults preserved)', async () => {
      mockLoadSettings.mockResolvedValue(null);

      render(TrayPanel);

      await vi.waitFor(() => {
        expect(mockLoadSettings).toHaveBeenCalledTimes(1);
      });

      // Wait a bit for any async operations to complete
      await new Promise(r => setTimeout(r, 100));

      // updateSettings should NOT be called from loadSavedSettings
      // (it may be called from $effect debounce, but the loadSavedSettings path should not call it)
      // Since loadSettings returned null, toTimerSettings should NOT be called from loadSavedSettings
      expect(mockToTimerSettings).not.toHaveBeenCalled();
    });
  });

  // =========================================================================
  // 12. handlePauseMediaChange (spec 7.4)
  // =========================================================================

  describe('handlePauseMediaChange', () => {
    it('12-1: savePauseMediaOnBreak is called when media pause toggle changes', async () => {
      mockLoadPauseMediaOnBreak.mockResolvedValue(false);

      render(TrayPanel);

      await vi.waitFor(() => {
        expect(mockGetTimerState).toHaveBeenCalled();
      });

      const label = screen.getByText('Auto-pause');
      const row = label.closest('.toggle-row');
      const toggle = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
      expect(toggle).toBeTruthy();
      await fireEvent.click(toggle!);

      await vi.waitFor(() => {
        expect(mockSavePauseMediaOnBreak).toHaveBeenCalledTimes(1);
        expect(mockSavePauseMediaOnBreak).toHaveBeenCalledWith(true);
      });
    });

    it('12-2: savePauseMediaOnBreak(false) is called when disabling media pause', async () => {
      mockLoadPauseMediaOnBreak.mockResolvedValue(true);

      render(TrayPanel);

      await vi.waitFor(() => {
        const label = screen.getByText('Auto-pause');
        const row = label.closest('.toggle-row');
        const toggle = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
        expect(toggle).toBeTruthy();
        expect(toggle.checked).toBe(true);
      });

      const label = screen.getByText('Auto-pause');
      const row = label.closest('.toggle-row');
      const toggle = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
      await fireEvent.click(toggle!);

      await vi.waitFor(() => {
        expect(mockSavePauseMediaOnBreak).toHaveBeenCalledTimes(1);
        expect(mockSavePauseMediaOnBreak).toHaveBeenCalledWith(false);
      });
    });
  });
});
