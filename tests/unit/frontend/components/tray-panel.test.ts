/**
 * TrayPanel - Unit Tests
 *
 * Tests for: frontend/components/TrayPanel.svelte
 * Spec: _documents/spec/frontend/components/tray-panel.md
 * Runtime: JS-ESM (Svelte 5)
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

vi.mock('@tauri-apps/api/dpi', () => ({
  LogicalSize: vi.fn(),
}));

// --- timer.ts mock ---

const {
  mockGetTimerState,
  mockTogglePause,
  mockUpdateSettings,
  mockQuitApp,
  mockOnTimerTick,
  mockRemainingSecs,
  mockFormatTime,
  mockResetTimer,
} = vi.hoisted(() => ({
  mockGetTimerState: vi.fn(),
  mockTogglePause: vi.fn(),
  mockUpdateSettings: vi.fn(),
  mockQuitApp: vi.fn(),
  mockOnTimerTick: vi.fn().mockResolvedValue(vi.fn()),
  mockRemainingSecs: vi.fn(),
  mockFormatTime: vi.fn(),
  mockResetTimer: vi.fn(),
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
  onTimerTick: mockOnTimerTick,
  onPhaseChanged: vi.fn().mockResolvedValue(vi.fn()),
  onBreakStart: vi.fn().mockResolvedValue(vi.fn()),
  onBreakEnd: vi.fn().mockResolvedValue(vi.fn()),
  getTodaySessions: vi.fn().mockResolvedValue(0),
  acceptBreak: vi.fn(),
  extendFocus: vi.fn(),
  skipBreakFromFocus: vi.fn(),
  resetTimer: mockResetTimer,
  setTrayIconVisible: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-autostart', () => ({
  isEnabled: vi.fn().mockResolvedValue(false),
  enable: vi.fn().mockResolvedValue(undefined),
  disable: vi.fn().mockResolvedValue(undefined),
}));

// --- settings-store mock ---

const {
  mockLoadSettings,
  mockSaveSettings,
  mockToTimerSettings,
  mockToDisplaySettings,
} = vi.hoisted(() => ({
  mockLoadSettings: vi.fn(),
  mockSaveSettings: vi.fn(),
  mockToTimerSettings: vi.fn(),
  mockToDisplaySettings: vi.fn(),
}));

vi.mock('@code/frontend/lib/settings-store', () => ({
  loadSettings: mockLoadSettings,
  saveSettings: mockSaveSettings,
  toTimerSettings: mockToTimerSettings,
  toDisplaySettings: mockToDisplaySettings,
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
  loadPresenceLevel: vi.fn().mockResolvedValue('dynamic'),
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

describe('TrayPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockRemainingSecs.mockReturnValue(1200);
    mockFormatTime.mockReturnValue('20:00');
    mockGetTimerState.mockResolvedValue(makeTimerState());
    mockLoadSettings.mockResolvedValue(null);
    mockTogglePause.mockResolvedValue(true);
    mockToTimerSettings.mockReturnValue({
      focus_duration_secs: 1200,
      short_break_duration_secs: 20,
      long_break_duration_secs: 180,
      short_breaks_before_long: 3,
    });
  });

  // =========================================================================
  // 1. Mount side effects
  // =========================================================================

  it('1-1: loadSettings is called on mount', async () => {
    render(TrayPanel);

    // Wait for full onMount to complete so no async leaks into next test
    await vi.waitFor(() => {
      expect(mockOnTimerTick).toHaveBeenCalled();
    });
    expect(mockLoadSettings).toHaveBeenCalledTimes(1);
  });

  it('1-2: getTimerState is called on mount', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockOnTimerTick).toHaveBeenCalled();
    });
    expect(mockGetTimerState).toHaveBeenCalledTimes(1);
  });

  it('1-3: onTimerTick event subscription is set up on mount', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockOnTimerTick).toHaveBeenCalledTimes(1);
      expect(typeof mockOnTimerTick.mock.calls[0][0]).toBe('function');
    });
  });

  // =========================================================================
  // 2. UI rendering
  // =========================================================================

  it('2-1: remaining timer display is rendered correctly', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('20:00')).toBeTruthy();
    });
  });

  it('2-2: tray-panel class container exists', async () => {
    const { container } = render(TrayPanel);

    await vi.waitFor(() => {
      const trayPanel = container.querySelector('.tray-panel');
      expect(trayPanel).toBeTruthy();
    });
  });

  it('2-3: stop button is displayed', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('■ Stop')).toBeTruthy();
    });
  });

  it('2-4: quit button is displayed', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('Quit')).toBeTruthy();
    });
  });

  // =========================================================================
  // 3. Settings save flow
  // =========================================================================

  it('3-1: after settings change, updateSettings and saveSettings are called via debounce', async () => {
    mockUpdateSettings.mockResolvedValue(undefined);
    mockSaveSettings.mockResolvedValue(undefined);

    render(TrayPanel);

    // Wait for settingsLoaded to become true
    await vi.waitFor(() => {
      expect(mockLoadSettings).toHaveBeenCalled();
    });

    // Wait for $effect initial fire to save
    await new Promise(r => setTimeout(r, 600));

    // Record call counts before change
    const updateCallsBefore = mockUpdateSettings.mock.calls.length;
    const saveCallsBefore = mockSaveSettings.mock.calls.length;

    // Change settings value (trigger via input event)
    const focusInput = document.getElementById('focus') as HTMLInputElement;
    await fireEvent.input(focusInput, { target: { value: '30' } });

    // Wait for additional save calls after debounce (500ms)
    await vi.waitFor(() => {
      expect(mockUpdateSettings.mock.calls.length).toBeGreaterThan(updateCallsBefore);
      expect(mockSaveSettings.mock.calls.length).toBeGreaterThan(saveCallsBefore);
    }, { timeout: 2000 });

    // Verify order of last call pair: updateSettings before saveSettings
    const lastUpdateOrder = mockUpdateSettings.mock.invocationCallOrder[mockUpdateSettings.mock.calls.length - 1];
    const lastSaveOrder = mockSaveSettings.mock.invocationCallOrder[mockSaveSettings.mock.calls.length - 1];
    expect(lastUpdateOrder).toBeLessThan(lastSaveOrder);
  });
});
