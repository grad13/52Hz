/**
 * SettingsForm - Supplement Tests
 *
 * Tests for: code/frontend/components/SettingsForm.svelte
 * Spec: documents/spec/frontend/components/settings-form.md
 * Runtime: JS-ESM (Svelte 5)
 *
 * Supplement for: tests/unit/frontend/components/settings-form.test.ts
 * Missing: 3-8 (自動起動トグル)
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
  onSave: vi.fn(),
  onAutostartChange: vi.fn(),
};

describe('SettingsForm - supplement', () => {
  // =========================================================================
  // 3-8: 自動起動トグル
  // =========================================================================

  it('3-8a: 自動起動トグルが「ログイン時に自動起動」ラベルで表示される', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByLabelText('ログイン時に自動起動')).toBeTruthy();
  });

  it('3-8b: 自動起動トグルの id="autostart"、type="checkbox"', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('autostart') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('checkbox');
  });

  it('3-8c: autostartEnabled=true のときチェック済み', () => {
    render(SettingsForm, { props: { ...defaultProps, autostartEnabled: true } });
    const input = document.getElementById('autostart') as HTMLInputElement;
    expect(input.checked).toBe(true);
  });

  it('3-8d: トグル変更で onAutostartChange が呼ばれる', async () => {
    const onAutostartChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onAutostartChange } });
    const input = document.getElementById('autostart') as HTMLInputElement;
    await fireEvent.click(input);
    expect(onAutostartChange).toHaveBeenCalledTimes(1);
  });
});
