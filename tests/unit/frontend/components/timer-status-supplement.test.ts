/**
 * TimerStatus - Supplement Tests (from spec-to-tests)
 *
 * Tests for: code/frontend/components/TimerStatus.svelte
 * Spec: _documents/spec/frontend/components/timer-status.md
 * Runtime: JS-ESM (Svelte 5)
 *
 * Supplement for: tests/unit/frontend/components/timer-status.test.ts
 * Missing: cycleTotal/cycleCompleted/isLongBreak props — cycle dots rendering
 */

import { describe, it, expect, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import TimerStatus from '@code/frontend/components/TimerStatus.svelte';

afterEach(() => { cleanup(); });

// --- Helpers ---

function renderWithCycleDots(overrides: Record<string, unknown> = {}) {
  return render(TimerStatus, {
    props: {
      phaseLabel: 'Focusing',
      remaining: '20:00',
      paused: false,
      cycleCompleted: 0,
      cycleTotal: 3,
      isLongBreak: false,
      ...overrides,
    },
  });
}

describe('TimerStatus - cycle dots (supplement)', () => {
  // =========================================================================
  // Spec 5.1: cycleTotal controls dot count
  // =========================================================================

  it('S1-1: cycleTotal=3 renders 3 .dot elements', () => {
    const { container } = renderWithCycleDots({ cycleTotal: 3 });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(3);
  });

  it('S1-2: cycleTotal=1 renders 1 .dot element', () => {
    const { container } = renderWithCycleDots({ cycleTotal: 1 });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(1);
  });

  it('S1-3: cycleTotal=0 renders no .dot elements', () => {
    const { container } = renderWithCycleDots({ cycleTotal: 0 });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(0);
  });

  // =========================================================================
  // Spec 5.2: cycleCompleted controls .filled class
  // Condition: i < cycleCompleted → .filled
  // =========================================================================

  it('S2-1: cycleCompleted=2, cycleTotal=3 fills first 2 dots', () => {
    const { container } = renderWithCycleDots({
      cycleTotal: 3,
      cycleCompleted: 2,
    });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(3);
    expect(dots[0].classList.contains('filled')).toBe(true);
    expect(dots[1].classList.contains('filled')).toBe(true);
    expect(dots[2].classList.contains('filled')).toBe(false);
  });

  it('S2-2: cycleCompleted=0, cycleTotal=3 has no .filled', () => {
    const { container } = renderWithCycleDots({
      cycleTotal: 3,
      cycleCompleted: 0,
    });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(3);
    dots.forEach(dot => {
      expect(dot.classList.contains('filled')).toBe(false);
    });
  });

  it('S2-3: cycleCompleted=3, cycleTotal=3 fills all dots', () => {
    const { container } = renderWithCycleDots({
      cycleTotal: 3,
      cycleCompleted: 3,
    });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(3);
    dots.forEach(dot => {
      expect(dot.classList.contains('filled')).toBe(true);
    });
  });

  // =========================================================================
  // Spec 5.2: isLongBreak=true → all dots filled
  // Condition: isLongBreak || i < cycleCompleted → .filled
  // =========================================================================

  it('S3-1: isLongBreak=true fills all dots regardless of cycleCompleted', () => {
    const { container } = renderWithCycleDots({
      cycleTotal: 3,
      cycleCompleted: 0,
      isLongBreak: true,
    });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(3);
    dots.forEach(dot => {
      expect(dot.classList.contains('filled')).toBe(true);
    });
  });

  it('S3-2: isLongBreak=false, cycleCompleted=1 partially fills dots', () => {
    const { container } = renderWithCycleDots({
      cycleTotal: 4,
      cycleCompleted: 1,
      isLongBreak: false,
    });

    const dots = container.querySelectorAll('.dot');
    expect(dots.length).toBe(4);
    expect(dots[0].classList.contains('filled')).toBe(true);
    expect(dots[1].classList.contains('filled')).toBe(false);
    expect(dots[2].classList.contains('filled')).toBe(false);
    expect(dots[3].classList.contains('filled')).toBe(false);
  });

  // =========================================================================
  // Spec 5.1: .cycle-dots container exists
  // =========================================================================

  it('S4-1: .cycle-dots container is rendered', () => {
    const { container } = renderWithCycleDots({ cycleTotal: 3 });

    const dotsContainer = container.querySelector('.cycle-dots');
    expect(dotsContainer).toBeTruthy();
  });
});
