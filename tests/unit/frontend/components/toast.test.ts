/**
 * Toast - Unit Tests
 *
 * Tests for: code/frontend/components/Toast.svelte
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

const { mockListen, mockEmit, mockWin, mockLoadPresenceToast, mockLoadPresencePosition, mockLoadPresenceLevel } = vi.hoisted(() => ({
  mockListen: vi.fn().mockResolvedValue(vi.fn()),
  mockEmit: vi.fn().mockResolvedValue(undefined),
  mockWin: {
    setSize: vi.fn().mockResolvedValue(undefined),
    show: vi.fn().mockResolvedValue(undefined),
    hide: vi.fn().mockResolvedValue(undefined),
  },
  mockLoadPresenceToast: vi.fn().mockResolvedValue(true),
  mockLoadPresencePosition: vi.fn().mockResolvedValue('top-right'),
  mockLoadPresenceLevel: vi.fn().mockResolvedValue('dynamic'),
}));

const { mockAcceptBreak, mockSkipBreakFromFocus } = vi.hoisted(() => ({
  mockAcceptBreak: vi.fn().mockResolvedValue(undefined),
  mockSkipBreakFromFocus: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen,
  emit: mockEmit,
}));
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => mockWin,
  currentMonitor: vi.fn().mockResolvedValue(null),
}));
vi.mock('@tauri-apps/api/dpi', () => ({
  LogicalSize: vi.fn(),
}));
vi.mock('@code/frontend/lib/settings-store', () => ({
  loadPresenceToast: mockLoadPresenceToast,
  loadPresencePosition: mockLoadPresencePosition,
  loadPresenceLevel: mockLoadPresenceLevel,
}));
vi.mock('@code/frontend/lib/timer', () => ({
  acceptBreak: mockAcceptBreak,
  skipBreakFromFocus: mockSkipBreakFromFocus,
}));

import Toast from '@code/frontend/components/Toast.svelte';

/** Helper: flush pending microtasks (awaits for listen/load calls in onMount) */
async function flushAsync() {
  await new Promise((r) => setTimeout(r, 0));
  await new Promise((r) => setTimeout(r, 0));
}

/** Helper: find listener callback by event name */
function getListenerCb(eventName: string) {
  const call = mockListen.mock.calls.find((c: unknown[]) => c[0] === eventName);
  if (!call) throw new Error(`No listener registered for "${eventName}"`);
  return call[1] as (event: { payload: unknown }) => void;
}

beforeEach(() => {
  vi.clearAllMocks();
  mockLoadPresenceToast.mockResolvedValue(true);
  mockLoadPresencePosition.mockResolvedValue('top-right');
  mockLoadPresenceLevel.mockResolvedValue('dynamic');
});

afterEach(() => {
  cleanup();
});

describe('Toast', () => {
  describe('Mounting', () => {
    it('renders without error', async () => {
      const { container } = render(Toast);
      await flushAsync();
      expect(container.querySelector('.toast-stack')).toBeTruthy();
    });

    it('registers 6 event listeners on mount', async () => {
      render(Toast);
      await flushAsync();
      expect(mockListen).toHaveBeenCalledTimes(6);
      const eventNames = mockListen.mock.calls.map((c: unknown[]) => c[0]);
      expect(eventNames).toContain('presence-message');
      expect(eventNames).toContain('presence-toast-toggle');
      expect(eventNames).toContain('presence-toast-click');
      expect(eventNames).toContain('focus-done-toast');
      expect(eventNames).toContain('presence-position-change');
      expect(eventNames).toContain('presence-level-setting');
    });

    it('loads initial settings on mount', async () => {
      render(Toast);
      await flushAsync();
      expect(mockLoadPresenceToast).toHaveBeenCalledTimes(1);
      expect(mockLoadPresencePosition).toHaveBeenCalledTimes(1);
    });
  });

  describe('Toast stack rendering', () => {
    it('toast message renders name and message text', async () => {
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Hello world' } });
      await flushAsync();

      const name = container.querySelector('.name');
      const msg = container.querySelector('.msg');
      expect(name?.textContent).toBe('Alice');
      expect(msg?.textContent).toBe('Hello world');
    });

    it('focus-done card renders session complete text and action buttons', async () => {
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('focus-done-toast');
      cb({ payload: undefined });
      await flushAsync();

      const label = container.querySelector('.label');
      expect(label?.textContent).toBe('セッション完了');

      const buttons = container.querySelectorAll('.actions .btn');
      expect(buttons.length).toBe(2);
      expect(buttons[0].textContent).toBe('休憩する');
      expect(buttons[1].textContent).toBe('スキップ');
    });
  });

  describe('Constants (MAX_TOASTS, WIN_W)', () => {
    it('adding 11 toasts evicts the oldest (MAX_TOASTS = 10)', async () => {
      render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      for (let i = 0; i < 11; i++) {
        cb({ payload: { name: `User${i}`, message: `Msg${i}` } });
      }
      await flushAsync();

      // Active (non-leaving) toasts should be at most 10
      const cards = document.querySelectorAll('.toast-card:not(.leaving)');
      expect(cards.length).toBeLessThanOrEqual(10);
    });

    it('WIN_W = 276 is passed to setSize', async () => {
      render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Test', message: 'Hi' } });
      await flushAsync();

      expect(mockWin.setSize).toHaveBeenCalled();
      // LogicalSize is mocked as vi.fn(), so we check it was called with 276 as first arg
      const { LogicalSize } = await import('@tauri-apps/api/dpi');
      expect(LogicalSize).toHaveBeenCalledWith(276, expect.any(Number));
    });
  });

  describe('Position-aware insertion', () => {
    it('top-right: new items are appended (last child is newest)', async () => {
      mockLoadPresencePosition.mockResolvedValue('top-right');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'First', message: 'M1' } });
      await flushAsync();
      cb({ payload: { name: 'Second', message: 'M2' } });
      await flushAsync();

      const cards = container.querySelectorAll('.toast-card');
      const names = Array.from(cards).map((c) => c.querySelector('.name')?.textContent);
      expect(names[0]).toBe('First');
      expect(names[1]).toBe('Second');
    });

    it('bottom-left: new items are prepended (first child is newest)', async () => {
      mockLoadPresencePosition.mockResolvedValue('bottom-left');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'First', message: 'M1' } });
      await flushAsync();
      cb({ payload: { name: 'Second', message: 'M2' } });
      await flushAsync();

      const cards = container.querySelectorAll('.toast-card');
      const names = Array.from(cards).map((c) => c.querySelector('.name')?.textContent);
      expect(names[0]).toBe('Second');
      expect(names[1]).toBe('First');
    });
  });
});
