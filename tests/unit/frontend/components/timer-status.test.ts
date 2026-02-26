/**
 * TimerStatus - Unit Tests
 *
 * Tests for: code/frontend/components/TimerStatus.svelte
 * Spec: documents/spec/frontend/components/timer-status.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import TimerStatus from '../../../../code/frontend/components/TimerStatus.svelte';

describe('TimerStatus', () => {
  it('1-1: phaseLabel が表示される', () => {
    render(TimerStatus, {
      props: { phaseLabel: 'フォーカス', remaining: '25:00', paused: false },
    });
    expect(screen.getByText('フォーカス')).toBeTruthy();
  });

  it('1-2: remaining が表示される', () => {
    render(TimerStatus, {
      props: { phaseLabel: 'フォーカス', remaining: '12:34', paused: false },
    });
    expect(screen.getByText('12:34')).toBeTruthy();
  });

  it('1-3: paused=true で一時停止中バッジ表示', () => {
    render(TimerStatus, {
      props: { phaseLabel: 'フォーカス', remaining: '25:00', paused: true },
    });
    expect(screen.getByText('一時停止中')).toBeTruthy();
  });

  it('1-4: paused=false でバッジ非表示', () => {
    render(TimerStatus, {
      props: { phaseLabel: 'フォーカス', remaining: '25:00', paused: false },
    });
    expect(screen.queryByText('一時停止中')).toBeNull();
  });
});
