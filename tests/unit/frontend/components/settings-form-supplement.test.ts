/**
 * SettingsForm - Supplement Tests
 *
 * Tests for: code/frontend/components/SettingsForm.svelte
 * Spec: documents/spec/frontend/components/settings-form.md
 * Runtime: JS-ESM (Svelte 5)
 *
 * Supplement for: tests/unit/frontend/components/settings-form.test.ts
 * Covers: 自動起動トグル, メディア自動中断トグル
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
  autostartEnabled: false,
  onAutostartChange: vi.fn(),
  pauseMediaOnBreak: false,
  onPauseMediaChange: vi.fn(),
  hideTrayIcon: false,
  onHideTrayIconChange: vi.fn(),
  tickVolume: 0,
  onTickVolumeChange: vi.fn(),
  presenceToast: true,
  onPresenceToastChange: vi.fn(),
  presencePosition: 'top-right' as const,
  onPresencePositionChange: vi.fn(),
  presenceLevel: 'front' as const,
  onPresenceLevelChange: vi.fn(),
};

function getToggleByLabel(labelText: string): HTMLInputElement {
  const label = screen.getByText(labelText);
  const row = label.closest('.toggle-row');
  return row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
}

describe('SettingsForm - supplement', () => {
  // =========================================================================
  // 3-8: 自動起動トグル
  // =========================================================================

  it('3-8a: 自動起動トグルが「自動起動」ラベルで表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('自動起動')).toBeTruthy();
  });

  it('3-8b: 自動起動トグルが checkbox である', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = getToggleByLabel('自動起動');
    expect(input).toBeTruthy();
    expect(input.type).toBe('checkbox');
  });

  it('3-8c: autostartEnabled=true のときチェック済み', () => {
    render(SettingsForm, { props: { ...defaultProps, autostartEnabled: true } });
    const input = getToggleByLabel('自動起動');
    expect(input.checked).toBe(true);
  });

  it('3-8d: トグル変更で onAutostartChange が呼ばれる', async () => {
    const onAutostartChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onAutostartChange } });
    const input = getToggleByLabel('自動起動');
    await fireEvent.click(input);
    expect(onAutostartChange).toHaveBeenCalledTimes(1);
  });

  // =========================================================================
  // 3-9: メディア自動中断トグル
  // =========================================================================

  it('3-9a: メディア自動中断トグルが「メディア自動中断」ラベルで表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('メディア自動中断')).toBeTruthy();
  });

  it('3-9b: メディア自動中断トグルが checkbox である', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = getToggleByLabel('メディア自動中断');
    expect(input).toBeTruthy();
    expect(input.type).toBe('checkbox');
  });

  it('3-9c: pauseMediaOnBreak=true のときチェック済み', () => {
    render(SettingsForm, { props: { ...defaultProps, pauseMediaOnBreak: true } });
    const input = getToggleByLabel('メディア自動中断');
    expect(input.checked).toBe(true);
  });

  it('3-9d: pauseMediaOnBreak=false のとき未チェック', () => {
    render(SettingsForm, { props: { ...defaultProps, pauseMediaOnBreak: false } });
    const input = getToggleByLabel('メディア自動中断');
    expect(input.checked).toBe(false);
  });

  it('3-9e: トグル変更で onPauseMediaChange が呼ばれる', async () => {
    const onPauseMediaChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onPauseMediaChange } });
    const input = getToggleByLabel('メディア自動中断');
    await fireEvent.click(input);
    expect(onPauseMediaChange).toHaveBeenCalledTimes(1);
  });
});
