/**
 * FocusDonePopup - Unit Tests
 *
 * Tests for: frontend/components/FocusDonePopup.svelte
 * Spec: _documents/spec/frontend/components/focus-done-popup.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import FocusDonePopup from '@code/frontend/components/FocusDonePopup.svelte';

afterEach(() => { cleanup(); });

// --- Hoisted mocks ---

const {
  mockClose,
  mockAcceptBreak,
  mockExtendFocus,
  mockSkipBreakFromFocus,
} = vi.hoisted(() => ({
  mockClose: vi.fn(),
  mockAcceptBreak: vi.fn(),
  mockExtendFocus: vi.fn(),
  mockSkipBreakFromFocus: vi.fn(),
}));

// --- Tauri API mocks ---

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({
    close: mockClose,
  }),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

// --- timer.ts mock ---

vi.mock('@code/frontend/lib/timer', () => ({
  remainingSecs: vi.fn(),
  formatTime: vi.fn(),
  getTimerState: vi.fn(),
  pauseTimer: vi.fn(),
  resumeTimer: vi.fn(),
  togglePause: vi.fn(),
  skipBreak: vi.fn(),
  updateSettings: vi.fn(),
  closeBreakOverlay: vi.fn(),
  quitApp: vi.fn(),
  onTimerTick: vi.fn().mockResolvedValue(vi.fn()),
  onPhaseChanged: vi.fn().mockResolvedValue(vi.fn()),
  onBreakStart: vi.fn().mockResolvedValue(vi.fn()),
  onBreakEnd: vi.fn().mockResolvedValue(vi.fn()),
  acceptBreak: mockAcceptBreak,
  extendFocus: mockExtendFocus,
  skipBreakFromFocus: mockSkipBreakFromFocus,
  getTodaySessions: vi.fn(),
}));

// --- settings-store mock ---

vi.mock('@code/frontend/lib/settings-store', () => ({
  loadSettings: vi.fn(),
  saveSettings: vi.fn(),
  toTimerSettings: vi.fn(),
  toDisplaySettings: vi.fn(),
}));

// --- Tests ---

describe('FocusDonePopup', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAcceptBreak.mockResolvedValue(undefined);
    mockExtendFocus.mockResolvedValue(undefined);
    mockSkipBreakFromFocus.mockResolvedValue(undefined);
  });

  // =========================================================================
  // 1. handleAcceptBreak (spec 3.1 / 5.3)
  // =========================================================================

  it('1-1: 「休憩する」ボタン押下で acceptBreak が呼ばれる', async () => {
    render(FocusDonePopup);

    const acceptButton = screen.getByText('休憩する');
    await fireEvent.click(acceptButton);

    await vi.waitFor(() => {
      expect(mockAcceptBreak).toHaveBeenCalledTimes(1);
    });
  });

  it('1-2: 「休憩する」ボタン押下後にウィンドウが閉じられる（IPC → close の順序保証）', async () => {
    render(FocusDonePopup);

    const acceptButton = screen.getByText('休憩する');
    await fireEvent.click(acceptButton);

    await vi.waitFor(() => {
      expect(mockClose).toHaveBeenCalledTimes(1);
      const acceptOrder = mockAcceptBreak.mock.invocationCallOrder[0];
      const closeOrder = mockClose.mock.invocationCallOrder[0];
      expect(acceptOrder).toBeLessThan(closeOrder);
    });
  });

  // =========================================================================
  // 2. handleSkip (spec 3.3 / 5.3)
  // =========================================================================

  it('2-1: 「スキップ」ボタン押下で skipBreakFromFocus が呼ばれる', async () => {
    render(FocusDonePopup);

    const skipButton = screen.getByText('スキップ');
    await fireEvent.click(skipButton);

    await vi.waitFor(() => {
      expect(mockSkipBreakFromFocus).toHaveBeenCalledTimes(1);
    });
  });

  it('2-2: 「スキップ」ボタン押下後にウィンドウが閉じられる（IPC → close の順序保証）', async () => {
    render(FocusDonePopup);

    const skipButton = screen.getByText('スキップ');
    await fireEvent.click(skipButton);

    await vi.waitFor(() => {
      expect(mockClose).toHaveBeenCalledTimes(1);
      const skipOrder = mockSkipBreakFromFocus.mock.invocationCallOrder[0];
      const closeOrder = mockClose.mock.invocationCallOrder[0];
      expect(skipOrder).toBeLessThan(closeOrder);
    });
  });

  // =========================================================================
  // 3. handleExtend (spec 3.2 / 5.3)
  // =========================================================================

  it('3-1: 「+1分」ボタン押下で extendFocus(60) が呼ばれる', async () => {
    render(FocusDonePopup);

    const btn = screen.getByText('+1分');
    await fireEvent.click(btn);

    await vi.waitFor(() => {
      expect(mockExtendFocus).toHaveBeenCalledWith(60);
    });
  });

  it('3-2: 「+3分」ボタン押下で extendFocus(180) が呼ばれる', async () => {
    render(FocusDonePopup);

    const btn = screen.getByText('+3分');
    await fireEvent.click(btn);

    await vi.waitFor(() => {
      expect(mockExtendFocus).toHaveBeenCalledWith(180);
    });
  });

  it('3-3: 「+5分」ボタン押下で extendFocus(300) が呼ばれる', async () => {
    render(FocusDonePopup);

    const btn = screen.getByText('+5分');
    await fireEvent.click(btn);

    await vi.waitFor(() => {
      expect(mockExtendFocus).toHaveBeenCalledWith(300);
    });
  });

  it('3-4: 延長ボタン押下後にウィンドウが閉じられる（IPC → close の順序保証）', async () => {
    render(FocusDonePopup);

    const btn = screen.getByText('+1分');
    await fireEvent.click(btn);

    await vi.waitFor(() => {
      expect(mockClose).toHaveBeenCalledTimes(1);
      const extendOrder = mockExtendFocus.mock.invocationCallOrder[0];
      const closeOrder = mockClose.mock.invocationCallOrder[0];
      expect(extendOrder).toBeLessThan(closeOrder);
    });
  });
});
