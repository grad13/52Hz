// meta: updated=2026-03-11 05:42 checked=-
/**
 * Toast - Unit Tests
 *
 * Tests for: code/frontend/components/Toast.svelte
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

const { mockListen, mockEmit, mockWin, mockLoadPresenceToast, mockLoadPresencePosition, mockLoadPresenceLevel, mockLoadPresenceMaxToasts, mockLoadPresenceShowIcon, mockLoadPresenceLikeIcon } = vi.hoisted(() => ({
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
  mockLoadPresenceMaxToasts: vi.fn().mockResolvedValue(4),
  mockLoadPresenceShowIcon: vi.fn().mockResolvedValue(true),
  mockLoadPresenceLikeIcon: vi.fn().mockResolvedValue('heart'),
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
  loadPresenceMaxToasts: mockLoadPresenceMaxToasts,
  loadPresenceShowIcon: mockLoadPresenceShowIcon,
  loadPresenceLikeIcon: mockLoadPresenceLikeIcon,
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

    it('registers 9 event listeners on mount', async () => {
      render(Toast);
      await flushAsync();
      expect(mockListen).toHaveBeenCalledTimes(9);
      const eventNames = mockListen.mock.calls.map((c: unknown[]) => c[0]);
      expect(eventNames).toContain('presence-message');
      expect(eventNames).toContain('presence-toast-toggle');
      expect(eventNames).toContain('presence-toast-click');
      expect(eventNames).toContain('focus-done-toast');
      expect(eventNames).toContain('presence-position-change');
      expect(eventNames).toContain('presence-level-setting');
      expect(eventNames).toContain('presence-max-toasts-change');
      expect(eventNames).toContain('presence-show-icon-change');
      expect(eventNames).toContain('presence-like-icon-change');
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
      expect(label?.textContent).toBe('Session complete');

      const buttons = container.querySelectorAll('.actions .btn');
      expect(buttons.length).toBe(5);
      expect(buttons[0].textContent).toBe('Take a break');
      expect(buttons[1].textContent).toBe('Skip');
      expect(buttons[2].textContent).toBe('+1m');
      expect(buttons[3].textContent).toBe('+3m');
      expect(buttons[4].textContent).toBe('+5m');
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

  describe('Z-order: focus-done does not raise to always-front', () => {
    it('focus-done in always-back mode does NOT emit presence-level-change with "always-front"', async () => {
      mockLoadPresenceLevel.mockResolvedValue('always-back');
      render(Toast);
      await flushAsync();

      mockEmit.mockClear();

      const cb = getListenerCb('focus-done-toast');
      cb({ payload: undefined });
      await flushAsync();

      // syncWindow emits presence-level-change to re-apply the current level after show(),
      // but it should NOT raise to "always-front" for focus-done.
      const raiseEmits = mockEmit.mock.calls.filter(
        (c: unknown[]) => c[0] === 'presence-level-change' && c[1] === 'always-front'
      );
      expect(raiseEmits.length).toBe(0);
    });

    it('focus-done in dynamic mode does NOT emit presence-level-change with "always-front"', async () => {
      mockLoadPresenceLevel.mockResolvedValue('dynamic');
      render(Toast);
      await flushAsync();

      mockEmit.mockClear();

      const cb = getListenerCb('focus-done-toast');
      cb({ payload: undefined });
      await flushAsync();

      // syncWindow emits presence-level-change to re-apply the current level after show(),
      // but it should NOT raise to "always-front" for focus-done.
      const raiseEmits = mockEmit.mock.calls.filter(
        (c: unknown[]) => c[0] === 'presence-level-change' && c[1] === 'always-front'
      );
      expect(raiseEmits.length).toBe(0);
    });
  });

  describe('Like feature', () => {
    it('like button is visible on toast cards by default (likeIcon=heart)', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('heart');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      const likeBtn = container.querySelector('.like-btn');
      expect(likeBtn).toBeTruthy();
      expect(likeBtn?.textContent).toBe('♥');
    });

    it('like button shows star when likeIcon=star', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('star');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      const likeBtn = container.querySelector('.like-btn');
      expect(likeBtn).toBeTruthy();
      expect(likeBtn?.textContent).toBe('★');
    });

    it('like button is hidden when likeIcon=none', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('none');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      const likeBtn = container.querySelector('.like-btn');
      expect(likeBtn).toBeNull();
    });

    it('clicking like button hides it (hasLikedThisSession guard)', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('heart');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      const likeBtn = container.querySelector('.like-btn') as HTMLButtonElement;
      expect(likeBtn).toBeTruthy();
      likeBtn.click();
      await flushAsync();

      // After liking, the like button should disappear (hasLikedThisSession = true)
      const likeBtnAfter = container.querySelector('.like-btn');
      expect(likeBtnAfter).toBeNull();
    });

    it('clicking like button shows bg-like art on the liked card', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('heart');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      const likeBtn = container.querySelector('.like-btn') as HTMLButtonElement;
      likeBtn.click();
      await flushAsync();

      const bgLike = container.querySelector('.bg-like');
      expect(bgLike).toBeTruthy();
      expect(bgLike?.textContent).toBe('♥');
    });

    it('second click on a different toast like button is ignored', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('heart');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Msg1' } });
      await flushAsync();
      cb({ payload: { name: 'Bob', message: 'Msg2' } });
      await flushAsync();

      // Click like on the first toast
      const likeBtns = container.querySelectorAll('.like-btn');
      expect(likeBtns.length).toBe(2);
      (likeBtns[0] as HTMLButtonElement).click();
      await flushAsync();

      // All like buttons should now be gone
      const likeBtnsAfter = container.querySelectorAll('.like-btn');
      expect(likeBtnsAfter.length).toBe(0);

      // Only one bg-like should exist (on the first card)
      const bgLikes = container.querySelectorAll('.bg-like');
      expect(bgLikes.length).toBe(1);
    });

    it('focus-done resets like state so like buttons reappear', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('heart');
      const { container } = render(Toast);
      await flushAsync();

      const msgCb = getListenerCb('presence-message');
      msgCb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      // Like the toast
      const likeBtn = container.querySelector('.like-btn') as HTMLButtonElement;
      likeBtn.click();
      await flushAsync();
      expect(container.querySelector('.like-btn')).toBeNull();

      // Add a new toast after liking (before focus-done) - should have no like button
      msgCb({ payload: { name: 'Bob', message: 'Hey' } });
      await flushAsync();
      expect(container.querySelectorAll('.like-btn').length).toBe(0);

      // Trigger focus-done to reset
      const focusDoneCb = getListenerCb('focus-done-toast');
      focusDoneCb({ payload: undefined });
      await flushAsync();

      // Add another toast - like button should be back
      msgCb({ payload: { name: 'Carol', message: 'World' } });
      await flushAsync();

      const likeBtnsAfter = container.querySelectorAll('.like-btn');
      expect(likeBtnsAfter.length).toBeGreaterThan(0);
    });

    it('bg-like shows star icon when likeIcon=star', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('star');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      const likeBtn = container.querySelector('.like-btn') as HTMLButtonElement;
      likeBtn.click();
      await flushAsync();

      const bgLike = container.querySelector('.bg-like');
      expect(bgLike).toBeTruthy();
      expect(bgLike?.classList.contains('star')).toBe(true);
      expect(bgLike?.textContent).toBe('★');
    });

    it('presence-like-icon-change event resets like state', async () => {
      mockLoadPresenceLikeIcon.mockResolvedValue('heart');
      const { container } = render(Toast);
      await flushAsync();

      const msgCb = getListenerCb('presence-message');
      msgCb({ payload: { name: 'Alice', message: 'Hi' } });
      await flushAsync();

      // Like a toast
      const likeBtn = container.querySelector('.like-btn') as HTMLButtonElement;
      likeBtn.click();
      await flushAsync();
      expect(container.querySelector('.like-btn')).toBeNull();

      // Change like icon setting -> should reset like state
      const likeIconCb = getListenerCb('presence-like-icon-change');
      likeIconCb({ payload: 'star' });
      await flushAsync();

      // Add new toast - like button should be available again
      msgCb({ payload: { name: 'Bob', message: 'Hey' } });
      await flushAsync();
      const likeBtnsAfter = container.querySelectorAll('.like-btn');
      expect(likeBtnsAfter.length).toBeGreaterThan(0);
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

    it('bottom-left: applies from-bottom and from-left CSS classes for visual reversal', async () => {
      mockLoadPresencePosition.mockResolvedValue('bottom-left');
      const { container } = render(Toast);
      await flushAsync();

      const cb = getListenerCb('presence-message');
      cb({ payload: { name: 'First', message: 'M1' } });
      await flushAsync();

      const stack = container.querySelector('.toast-stack');
      expect(stack?.classList.contains('from-bottom')).toBe(true);
      expect(stack?.classList.contains('from-left')).toBe(true);
    });
  });
});
