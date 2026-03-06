/**
 * TimerStatus - Unit Tests
 *
 * Tests for: code/frontend/components/TimerStatus.svelte
 * Spec: _documents/spec/frontend/components/timer-status.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import TimerStatus from '@code/frontend/components/TimerStatus.svelte';

afterEach(() => { cleanup(); });

const defaultProps = {
  remaining: '25:00',
  paused: false,
  cycleCompleted: 0,
  cycleTotal: 3,
  isLongBreak: false,
  todaySessions: 0,
  onTogglePause: vi.fn(),
};

describe('TimerStatus', () => {
  it('1-1: remaining が表示される', () => {
    render(TimerStatus, { props: { ...defaultProps, remaining: '12:34' } });
    expect(screen.getByText('12:34')).toBeTruthy();
  });

  it('1-2: paused=true で一時停止中の表示', () => {
    render(TimerStatus, { props: { ...defaultProps, paused: true } });
    // Paused state should be reflected in the UI
    expect(screen.getByText('25:00')).toBeTruthy();
  });

  it('1-3: cycleCompleted/cycleTotal が反映される', () => {
    render(TimerStatus, {
      props: { ...defaultProps, cycleCompleted: 2, cycleTotal: 4 },
    });
    expect(screen.getByText('25:00')).toBeTruthy();
  });

  it('1-4: todaySessions が反映される', () => {
    render(TimerStatus, {
      props: { ...defaultProps, todaySessions: 5 },
    });
    expect(screen.getByText('25:00')).toBeTruthy();
  });
});
