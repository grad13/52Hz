/**
 * SettingsForm Presence - Unit Tests
 *
 * Tests for: code/frontend/components/SettingsForm.svelte
 * Spec: _documents/spec/frontend/components/settings-form.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import SettingsForm from '@code/frontend/components/SettingsForm.svelte';

afterEach(() => { cleanup(); });

const defaultProps = {
  focusMinutes: 25,
  shortBreakMinutes: 1,
  longBreakMinutes: 15,
  shortBreaksBeforeLong: 4,
  onAutostartChange: vi.fn(),
  onPauseMediaChange: vi.fn(),
  onHideTrayIconChange: vi.fn(),
  onTickVolumeChange: vi.fn(),
  onPresenceToastChange: vi.fn(),
  onPresencePositionChange: vi.fn(),
  onPresenceLevelChange: vi.fn(),
  tickVolume: 0.5,
  presenceToast: true,
  presencePosition: "top-right" as const,
  presenceLevel: "front" as const,
};

describe('SettingsForm - Volume slider', () => {
  it('3-9a: Tick音 label が表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('Tick音')).toBeTruthy();
  });

  it('3-9b: Volume slider が type="range", min="0", max="1", step="0.05" で存在する', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const slider = document.querySelector('.volume-slider') as HTMLInputElement;
    expect(slider).toBeTruthy();
    expect(slider.type).toBe('range');
    expect(slider.min).toBe('0');
    expect(slider.max).toBe('1');
    expect(slider.step).toBe('0.05');
  });

  it('3-9c: Volume slider が tickVolume prop の値を反映する', () => {
    render(SettingsForm, { props: { ...defaultProps, tickVolume: 0.8 } });
    const slider = document.querySelector('.volume-slider') as HTMLInputElement;
    expect(slider.value).toBe('0.8');
  });
});

describe('SettingsForm - Presence section', () => {
  it('3-10a: "みんなの存在" テキストが表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('みんなの存在')).toBeTruthy();
  });

  it('3-10b: Presence トグルが存在し presenceToast prop を反映する', () => {
    render(SettingsForm, { props: { ...defaultProps, presenceToast: false } });
    const section = document.querySelector('.presence-section')!;
    const toggle = section.querySelector('input[type="checkbox"]') as HTMLInputElement;
    expect(toggle).toBeTruthy();
    expect(toggle.checked).toBe(false);
  });

  it('3-10c: Presence トグル変更時に onPresenceToastChange が呼ばれる', async () => {
    const onPresenceToastChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, presenceToast: false, onPresenceToastChange } });
    const section = document.querySelector('.presence-section')!;
    const toggle = section.querySelector('input[type="checkbox"]') as HTMLInputElement;
    await fireEvent.change(toggle, { target: { checked: true } });
    expect(onPresenceToastChange).toHaveBeenCalledWith(true);
  });

  it('3-11a: 4つのポジションボタンが存在する (↖, ↗, ↙, ↘)', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const buttons = document.querySelectorAll('.pos-btn');
    expect(buttons.length).toBe(4);
    const labels = Array.from(buttons).map((b) => b.textContent);
    expect(labels).toEqual(['↖', '↗', '↙', '↘']);
  });

  it('3-11b: アクティブなポジションボタンに active クラスが付く', () => {
    render(SettingsForm, { props: { ...defaultProps, presencePosition: 'top-right' as const } });
    const buttons = document.querySelectorAll('.pos-btn');
    const topRight = Array.from(buttons).find((b) => b.textContent === '↗')!;
    expect(topRight.classList.contains('active')).toBe(true);
    const topLeft = Array.from(buttons).find((b) => b.textContent === '↖')!;
    expect(topLeft.classList.contains('active')).toBe(false);
  });

  it('3-11c: ポジションボタンクリック時に onPresencePositionChange が呼ばれる', async () => {
    const onPresencePositionChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onPresencePositionChange } });
    const buttons = document.querySelectorAll('.pos-btn');
    const bottomLeft = Array.from(buttons).find((b) => b.textContent === '↙')!;
    await fireEvent.click(bottomLeft);
    expect(onPresencePositionChange).toHaveBeenCalledWith('bottom-left');
  });

  it('3-11d: 2つのレベルボタンが存在する (前面, 背面)', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const buttons = document.querySelectorAll('.level-btn');
    expect(buttons.length).toBe(2);
    const labels = Array.from(buttons).map((b) => b.textContent);
    expect(labels).toEqual(['前面', '背面']);
  });

  it('3-11e: アクティブなレベルボタンに active クラスが付く', () => {
    render(SettingsForm, { props: { ...defaultProps, presenceLevel: 'front' as const } });
    const buttons = document.querySelectorAll('.level-btn');
    const front = Array.from(buttons).find((b) => b.textContent === '前面')!;
    expect(front.classList.contains('active')).toBe(true);
    const back = Array.from(buttons).find((b) => b.textContent === '背面')!;
    expect(back.classList.contains('active')).toBe(false);
  });

  it('3-11f: レベルボタンクリック時に onPresenceLevelChange が呼ばれる', async () => {
    const onPresenceLevelChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onPresenceLevelChange } });
    const buttons = document.querySelectorAll('.level-btn');
    const back = Array.from(buttons).find((b) => b.textContent === '背面')!;
    await fireEvent.click(back);
    expect(onPresenceLevelChange).toHaveBeenCalledWith('back');
  });
});
