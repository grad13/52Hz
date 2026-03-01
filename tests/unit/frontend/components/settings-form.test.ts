/**
 * SettingsForm - Unit Tests
 *
 * Tests for: code/frontend/components/SettingsForm.svelte
 * Spec: documents/spec/frontend/components/settings-form.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import SettingsForm from '@code/frontend/components/SettingsForm.svelte';

afterEach(() => { cleanup(); });

const defaultProps = {
  focusMinutes: 25,
  shortBreakSecs: 20,
  longBreakMinutes: 15,
  shortBreaksBeforeLong: 4,
  onSave: vi.fn(),
};

describe('SettingsForm', () => {
  it('3-1: 4つの入力フィールドが表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByLabelText('フォーカス時間')).toBeTruthy();
    expect(screen.getByLabelText('短い休憩')).toBeTruthy();
    expect(screen.getByLabelText('長い休憩')).toBeTruthy();
    expect(screen.getByLabelText('長い休憩までの回数')).toBeTruthy();
  });

  it('3-2: 保存ボタン "設定を保存" が表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('設定を保存')).toBeTruthy();
  });

  it('3-3: 保存ボタンクリックで onSave が呼ばれる', async () => {
    const onSave = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onSave } });
    await fireEvent.click(screen.getByText('設定を保存'));
    expect(onSave).toHaveBeenCalledTimes(1);
  });

  it('3-4: フォーカス時間フィールドの min=1, max=120', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('focus') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('120');
  });

  it('3-5: 短い休憩フィールドの min=5, max=300', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('short-break') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('5');
    expect(input.max).toBe('300');
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
});
