/**
 * TrayPanel - Unit Tests
 *
 * Tests for: frontend/components/TrayPanel.svelte
 * Spec: documents/spec/frontend/components/tray-panel.md
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
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({
    close: vi.fn(),
  }),
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

  it('3-1: "52Hz" ヘッダーが表示される', async () => {
    render(TrayPanel);

    await vi.waitFor(() => {
      expect(screen.getByText('52Hz')).toBeTruthy();
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

  it('4-1: 設定変更後に debounce で updateSettings → saveSettings が呼ばれる', async () => {
    mockUpdateSettings.mockResolvedValue(undefined);
    mockSaveSettings.mockResolvedValue(undefined);

    render(TrayPanel);

    // settingsLoaded が true になるのを待つ
    await vi.waitFor(() => {
      expect(mockLoadSettings).toHaveBeenCalled();
    });

    // $effect の初回発火による保存を待つ
    await new Promise(r => setTimeout(r, 600));

    // 変更前の呼び出し回数を記録
    const updateCallsBefore = mockUpdateSettings.mock.calls.length;
    const saveCallsBefore = mockSaveSettings.mock.calls.length;

    // 設定値を変更（input イベントでトリガー）
    const focusInput = document.getElementById('focus') as HTMLInputElement;
    await fireEvent.input(focusInput, { target: { value: '30' } });

    // debounce (500ms) 後に追加の保存が呼ばれるのを待つ
    await vi.waitFor(() => {
      expect(mockUpdateSettings.mock.calls.length).toBeGreaterThan(updateCallsBefore);
      expect(mockSaveSettings.mock.calls.length).toBeGreaterThan(saveCallsBefore);
    }, { timeout: 2000 });

    // 最後の呼び出しペアで順序確認: updateSettings が saveSettings より先
    const lastUpdateOrder = mockUpdateSettings.mock.invocationCallOrder[mockUpdateSettings.mock.calls.length - 1];
    const lastSaveOrder = mockSaveSettings.mock.invocationCallOrder[mockSaveSettings.mock.calls.length - 1];
    expect(lastUpdateOrder).toBeLessThan(lastSaveOrder);
  });
});
