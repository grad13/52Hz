/**
 * BreakOverlay - Unit Tests
 *
 * Tests for: frontend/components/BreakOverlay.svelte
 * Spec: _documents/spec/frontend/components/break-overlay.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import { mock } from 'vitest-mock-extended';
import type { Window as TauriWindow } from '@tauri-apps/api/window';
import type { TimerState } from '@code/frontend/lib/timer';
import BreakOverlay from '@code/frontend/components/BreakOverlay.svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';

afterEach(() => { cleanup(); });

// --- Hoisted mocks (vi.mock factories are hoisted, so variables must be too) ---

const {
  mockGetTimerState,
  mockSkipBreak,
  mockOnTimerTick,
  mockOnBreakEnd,
  mockRemainingSecs,
  mockFormatTime,
} = vi.hoisted(() => ({
  mockGetTimerState: vi.fn(),
  mockSkipBreak: vi.fn(),
  mockOnTimerTick: vi.fn().mockResolvedValue(vi.fn()),
  mockOnBreakEnd: vi.fn().mockResolvedValue(vi.fn()),
  mockRemainingSecs: vi.fn(),
  mockFormatTime: vi.fn(),
}));

// --- Tauri API mocks ---

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

// --- timer.ts mock ---

vi.mock('@code/frontend/lib/timer', () => ({
  remainingSecs: mockRemainingSecs,
  formatTime: mockFormatTime,
  getTimerState: mockGetTimerState,
  pauseTimer: vi.fn(),
  resumeTimer: vi.fn(),
  togglePause: vi.fn(),
  skipBreak: mockSkipBreak,
  updateSettings: vi.fn(),
  closeBreakOverlay: vi.fn(),
  quitApp: vi.fn(),
  onTimerTick: mockOnTimerTick,
  onPhaseChanged: vi.fn().mockResolvedValue(vi.fn()),
  onBreakStart: vi.fn().mockResolvedValue(vi.fn()),
  onBreakEnd: mockOnBreakEnd,
}));

// --- settings-store mock ---

vi.mock('@code/frontend/lib/settings-store', () => ({
  loadSettings: vi.fn(),
  saveSettings: vi.fn(),
  toTimerSettings: vi.fn(),
  toDisplaySettings: vi.fn(),
}));

// --- Type-safe mocks ---

const mockWindowInstance = mock<TauriWindow>();

// --- Helpers ---

function makeTimerState(overrides: Partial<TimerState> = {}): TimerState {
  return {
    phase: 'ShortBreak',
    paused: false,
    elapsed_secs: 0,
    phase_duration_secs: 20,
    short_break_count: 1,
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

describe('BreakOverlay', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getCurrentWindow).mockReturnValue(mockWindowInstance);
    mockRemainingSecs.mockReturnValue(20);
    mockFormatTime.mockReturnValue('00:20');
    mockGetTimerState.mockResolvedValue(makeTimerState());
  });

  // =========================================================================
  // 1. Mount side effects
  // =========================================================================

  it('1-1: getTimerState is called on mount', async () => {
    render(BreakOverlay);

    // getTimerState is called during onMount (async)
    await vi.waitFor(() => {
      expect(mockGetTimerState).toHaveBeenCalledTimes(1);
    });
  });

  it('1-2: onTimerTick event listener is registered on mount', async () => {
    render(BreakOverlay);

    await vi.waitFor(() => {
      expect(mockOnTimerTick).toHaveBeenCalledTimes(1);
      expect(typeof mockOnTimerTick.mock.calls[0][0]).toBe('function');
    });
  });

  it('1-3: onBreakEnd event listener is registered on mount', async () => {
    render(BreakOverlay);

    await vi.waitFor(() => {
      expect(mockOnBreakEnd).toHaveBeenCalledTimes(1);
      expect(typeof mockOnBreakEnd.mock.calls[0][0]).toBe('function');
    });
  });

  // =========================================================================
  // 2. Phase message display
  // =========================================================================

  it('2-1: ShortBreak phase shows "Rest your eyes"', async () => {
    mockGetTimerState.mockResolvedValue(
      makeTimerState({ phase: 'ShortBreak', elapsed_secs: 0, phase_duration_secs: 20 }),
    );
    mockRemainingSecs.mockReturnValue(20);
    mockFormatTime.mockReturnValue('00:20');

    render(BreakOverlay);

    await vi.waitFor(() => {
      expect(screen.getByText('Rest your eyes')).toBeTruthy();
      expect(screen.getByText('Look away and blink')).toBeTruthy();
    });
  });

  it('2-2: LongBreak phase shows "Stand up and stretch"', async () => {
    mockGetTimerState.mockResolvedValue(
      makeTimerState({ phase: 'LongBreak', elapsed_secs: 0, phase_duration_secs: 180 }),
    );
    mockRemainingSecs.mockReturnValue(180);
    mockFormatTime.mockReturnValue('03:00');

    render(BreakOverlay);

    await vi.waitFor(() => {
      expect(screen.getByText('Stand up and stretch')).toBeTruthy();
      expect(screen.getByText('Move around and take a deep breath')).toBeTruthy();
    });
  });

  it('2-3: Focus phase shows fallback "Break time"', async () => {
    // Focus phase has empty strings in the messages map, falls back to "Break time"
    mockGetTimerState.mockResolvedValue(
      makeTimerState({ phase: 'Focus', elapsed_secs: 0, phase_duration_secs: 1200 }),
    );
    mockRemainingSecs.mockReturnValue(1200);
    mockFormatTime.mockReturnValue('20:00');

    render(BreakOverlay);

    await vi.waitFor(() => {
      expect(screen.getByText('Break time')).toBeTruthy();
    });
  });

  // =========================================================================
  // 3. User interaction & events
  // =========================================================================

  it('3-1: clicking skip button calls skipBreak', async () => {
    render(BreakOverlay);

    await vi.waitFor(() => {
      expect(mockGetTimerState).toHaveBeenCalled();
    });

    const skipButton = screen.getByRole('button');
    await fireEvent.click(skipButton);

    expect(mockSkipBreak).toHaveBeenCalledTimes(1);
  });

  it('3-2: getCurrentWindow().close() is called on break-end event', async () => {
    render(BreakOverlay);

    await vi.waitFor(() => {
      expect(mockOnBreakEnd).toHaveBeenCalledTimes(1);
    });

    // Retrieve the handleBreakEnd callback registered with onBreakEnd
    const handleBreakEnd = mockOnBreakEnd.mock.calls[0][0];

    // Simulate receiving the break-end event
    handleBreakEnd();

    expect(mockWindowInstance.close).toHaveBeenCalledTimes(1);
  });

  // =========================================================================
  // 4. Visibility / CSS class
  // =========================================================================

  it('4-1: .visible class is not applied when initialized=false', () => {
    // Make getTimerState never resolve so initialized stays false
    mockGetTimerState.mockReturnValue(new Promise(() => {}));

    const { container } = render(BreakOverlay);
    const overlay = container.querySelector('.overlay');

    expect(overlay).toBeTruthy();
    expect(overlay!.classList.contains('visible')).toBe(false);
  });

  it('4-2: .visible class is applied after initial timer state is fetched (initialized=true)', async () => {
    mockGetTimerState.mockResolvedValue(makeTimerState());

    const { container } = render(BreakOverlay);

    await vi.waitFor(() => {
      const overlay = container.querySelector('.overlay');
      expect(overlay).toBeTruthy();
      expect(overlay!.classList.contains('visible')).toBe(true);
    });
  });
});
