/**
 * SettingsForm - Unit Tests
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

describe('SettingsForm', () => {
  it('3-1: 4つの入力フィールドが表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByLabelText('フォーカス')).toBeTruthy();
    expect(screen.getByLabelText('短い休憩')).toBeTruthy();
    expect(screen.getByLabelText('長い休憩')).toBeTruthy();
    expect(screen.getByLabelText('サイクル')).toBeTruthy();
  });

  it('3-4: フォーカス時間フィールドの min=1, max=120', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('focus') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('120');
  });

  it('3-5: 短い休憩フィールドの min=1, max属性なし', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('short-break') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('');
  });

  it('3-6: 長い休憩フィールドの min=1, max=30', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('long-break') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('30');
  });

  it('3-7: 長い休憩までの回数フィールドの min=1, max=10', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('cycles') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('10');
  });

  it('3-8: アイコン非表示トグルが表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('アイコン非表示')).toBeTruthy();
  });

  it('3-8b: hideTrayIcon=true のときアイコン非表示トグルがチェック済み', () => {
    const { container } = render(SettingsForm, { props: { ...defaultProps, hideTrayIcon: true } });
    const label = screen.getByText('アイコン非表示');
    const row = label.closest('.toggle-row');
    const input = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.checked).toBe(true);
  });

  it('3-8c: アイコン非表示トグル変更時に onHideTrayIconChange が呼ばれる', async () => {
    const onHideTrayIconChange = vi.fn();
    const { container } = render(SettingsForm, { props: { ...defaultProps, onHideTrayIconChange } });
    const label = screen.getByText('アイコン非表示');
    const row = label.closest('.toggle-row');
    const input = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
    await fireEvent.click(input);
    expect(onHideTrayIconChange).toHaveBeenCalledTimes(1);
  });
});
