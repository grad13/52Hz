/**
 * TimerControls - Unit Tests
 *
 * Tests for: code/frontend/components/TimerControls.svelte
 * Spec: documents/spec/frontend/components/timer-controls.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import TimerControls from '@code/frontend/components/TimerControls.svelte';

afterEach(() => { cleanup(); });

describe('TimerControls', () => {
  it('2-1: paused=false のとき "⏸ 一時停止" が表示される', () => {
    render(TimerControls, {
      props: { paused: false, onTogglePause: vi.fn(), onQuit: vi.fn() },
    });
    expect(screen.getByText('⏸ 一時停止')).toBeTruthy();
  });

  it('2-2: paused=true のとき "▶ 再開" が表示される', () => {
    render(TimerControls, {
      props: { paused: true, onTogglePause: vi.fn(), onQuit: vi.fn() },
    });
    expect(screen.getByText('▶ 再開')).toBeTruthy();
  });

  it('2-3: 終了ボタンに "✕ 終了" が常に表示される', () => {
    render(TimerControls, {
      props: { paused: false, onTogglePause: vi.fn(), onQuit: vi.fn() },
    });
    expect(screen.getByText('✕ 終了')).toBeTruthy();
  });

  it('2-4: トグルボタンクリックで onTogglePause が呼ばれる', async () => {
    const onTogglePause = vi.fn();
    render(TimerControls, {
      props: { paused: false, onTogglePause, onQuit: vi.fn() },
    });
    await fireEvent.click(screen.getByText('⏸ 一時停止'));
    expect(onTogglePause).toHaveBeenCalledTimes(1);
  });

  it('2-5: 終了ボタンクリックで onQuit が呼ばれる', async () => {
    const onQuit = vi.fn();
    render(TimerControls, {
      props: { paused: false, onTogglePause: vi.fn(), onQuit },
    });
    await fireEvent.click(screen.getByText('✕ 終了'));
    expect(onQuit).toHaveBeenCalledTimes(1);
  });

  it('2-6: ボタンが2つ描画される', () => {
    render(TimerControls, {
      props: { paused: false, onTogglePause: vi.fn(), onQuit: vi.fn() },
    });
    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(2);
  });
});
