/**
 * TrayPanel - Unit Tests
 *
 * Tests for: frontend/components/TrayPanel.svelte
 * Spec: documents/spec/frontend/components/tray-panel.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import TrayPanel from '../../../../code/frontend/components/TrayPanel.svelte';

// --- Tauri API mocks ---

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({
    close: vi.fn(),
  }),
}));

// --- timer.ts mock ---

const mockGetTimerState = vi.fn();
const mockTogglePause = vi.fn();
const mockUpdateSettings = vi.fn();
const mockQuitApp = vi.fn();
const mockOnTimerTick = vi.fn().mockResolvedValue(vi.fn());
const mockRemainingSecs = vi.fn();
const mockFormatTime = vi.fn();

vi.mock('../../../../code/frontend/lib/timer', () => ({
  remainingSecs: (...args: unknown[]) => mockRemainingSecs(...args),
  formatTime: (...args: unknown[]) => mockFormatTime(...args),
  getTimerState: (...args: unknown[]) => mockGetTimerState(...args),
  pauseTimer: vi.fn(),
  resumeTimer: vi.fn(),
  togglePause: (...args: unknown[]) => mockTogglePause(...args),
  skipBreak: vi.fn(),
  updateSettings: (...args: unknown[]) => mockUpdateSettings(...args),
  closeBreakOverlay: vi.fn(),
  quitApp: (...args: unknown[]) => mockQuitApp(...args),
  onTimerTick: (...args: unknown[]) => mockOnTimerTick(...args),
  onPhaseChanged: vi.fn().mockResolvedValue(vi.fn()),
  onBreakStart: vi.fn().mockResolvedValue(vi.fn()),
  onBreakEnd: vi.fn().mockResolvedValue(vi.fn()),
}));

// --- settings-store mock ---

const mockLoadSettings = vi.fn();
const mockSaveSettings = vi.fn();
const mockToTimerSettings = vi.fn();
const mockToDisplaySettings = vi.fn();

vi.mock('../../../../code/frontend/lib/settings-store', () => ({
  loadSettings: (...args: unknown[]) => mockLoadSettings(...args),
  saveSettings: (...args: unknown[]) => mockSaveSettings(...args),
  toTimerSettings: (...args: unknown[]) => mockToTimerSettings(...args),
  toDisplaySettings: (...args: unknown[]) => mockToDisplaySettings(...args),
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

  it('1-1: マウント時に loadSettings が呼ばれる', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockLoadSettings).toHaveBeenCalledTimes(1);
    });
  });

  it('1-2: マウント時に getTimerState が呼ばれる', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockGetTimerState).toHaveBeenCalledTimes(1);
    });
  });

  it('1-3: マウント時に onTimerTick でイベント購読される', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockOnTimerTick).toHaveBeenCalledTimes(1);
      expect(typeof mockOnTimerTick.mock.calls[0][0]).toBe('function');
    });
  });

  // =========================================================================
  // 2. Phase label mapping
  // =========================================================================

  it('2-1: フェーズラベルマッピング - Focus → "フォーカス中"', async () => {
    mockGetTimerState.mockResolvedValue(makeTimerState({ phase: 'Focus' }));

    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('フォーカス中')).toBeTruthy();
    });
  });

  it('2-2: フェーズラベルマッピング - ShortBreak → "短い休憩中"', async () => {
    mockGetTimerState.mockResolvedValue(
      makeTimerState({ phase: 'ShortBreak', phase_duration_secs: 20 }),
    );
    mockRemainingSecs.mockReturnValue(20);
    mockFormatTime.mockReturnValue('00:20');

    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('短い休憩中')).toBeTruthy();
    });
  });

  it('2-3: フェーズラベルマッピング - LongBreak → "長い休憩中"', async () => {
    mockGetTimerState.mockResolvedValue(
      makeTimerState({ phase: 'LongBreak', phase_duration_secs: 180 }),
    );
    mockRemainingSecs.mockReturnValue(180);
    mockFormatTime.mockReturnValue('03:00');

    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('長い休憩中')).toBeTruthy();
    });
  });

  it('2-4: 未知のフェーズ → そのまま表示（フォールバック）', async () => {
    mockGetTimerState.mockResolvedValue(
      makeTimerState({ phase: 'UnknownPhase' }),
    );

    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('UnknownPhase')).toBeTruthy();
    });
  });

  // =========================================================================
  // 3. UI rendering
  // =========================================================================

  it('3-1: "RestRun" ヘッダーが表示される', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('RestRun')).toBeTruthy();
    });
  });

  it('3-2: TimerStatus, TimerControls, SettingsForm が子コンポーネントとしてマウントされる', async () => {
    const { container } = render(TrayPanel);

    await vi.waitFor(() => {
      // TimerStatus renders phase label and remaining time
      expect(screen.getByText('フォーカス中')).toBeTruthy();
      expect(screen.getByText('20:00')).toBeTruthy();

      // TimerControls renders pause/quit buttons
      // SettingsForm renders the settings form
      // Verify the tray-panel structure contains expected child sections
      const trayPanel = container.querySelector('.tray-panel');
      expect(trayPanel).toBeTruthy();
      expect(trayPanel!.children.length).toBeGreaterThanOrEqual(3);
    });
  });

  // =========================================================================
  // 4. Settings save flow
  // =========================================================================

  it('4-1: handleSaveSettings が updateSettings → saveSettings の順で呼ばれる', async () => {
    mockUpdateSettings.mockResolvedValue(undefined);
    mockSaveSettings.mockResolvedValue(undefined);

    render(TrayPanel);

    await vi.waitFor(() => {
      expect(mockGetTimerState).toHaveBeenCalled();
    });

    // Find and click the save button (SettingsForm emits onSave callback)
    const saveButton = screen.getByRole('button', { name: /保存/i });
    await fireEvent.click(saveButton);

    await vi.waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledTimes(1);
      expect(mockSaveSettings).toHaveBeenCalledTimes(1);

      // Verify order: updateSettings is called before saveSettings
      const updateCallOrder = mockUpdateSettings.mock.invocationCallOrder[0];
      const saveCallOrder = mockSaveSettings.mock.invocationCallOrder[0];
      expect(updateCallOrder).toBeLessThan(saveCallOrder);
    });
  });
});
