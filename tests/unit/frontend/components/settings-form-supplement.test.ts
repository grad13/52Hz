/**
 * SettingsForm - Supplement Tests
 *
 * Tests for: code/frontend/components/SettingsForm.svelte
 * Spec: _documents/spec/frontend/components/settings-form.md
 * Runtime: JS-ESM (Svelte 5)
 *
 * Supplement for: tests/unit/frontend/components/settings-form.test.ts
 * Covers: autostart toggle, auto-pause media toggle
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
  // 3-8: Autostart toggle
  // =========================================================================

  it('3-8a: autostart toggle is displayed with "Launch at login" label', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('Launch at login')).toBeTruthy();
  });

  it('3-8b: autostart toggle is a checkbox', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = getToggleByLabel('Launch at login');
    expect(input).toBeTruthy();
    expect(input.type).toBe('checkbox');
  });

  it('3-8c: checked when autostartEnabled=true', () => {
    render(SettingsForm, { props: { ...defaultProps, autostartEnabled: true } });
    const input = getToggleByLabel('Launch at login');
    expect(input.checked).toBe(true);
  });

  it('3-8d: onAutostartChange is called when toggle changes', async () => {
    const onAutostartChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onAutostartChange } });
    const input = getToggleByLabel('Launch at login');
    await fireEvent.click(input);
    expect(onAutostartChange).toHaveBeenCalledTimes(1);
  });

  // =========================================================================
  // 3-9: Auto-pause media toggle
  // =========================================================================

  it('3-9a: auto-pause toggle is displayed with "Auto-pause" label', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('Auto-pause')).toBeTruthy();
  });

  it('3-9b: auto-pause toggle is a checkbox', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = getToggleByLabel('Auto-pause');
    expect(input).toBeTruthy();
    expect(input.type).toBe('checkbox');
  });

  it('3-9c: checked when pauseMediaOnBreak=true', () => {
    render(SettingsForm, { props: { ...defaultProps, pauseMediaOnBreak: true } });
    const input = getToggleByLabel('Auto-pause');
    expect(input.checked).toBe(true);
  });

  it('3-9d: unchecked when pauseMediaOnBreak=false', () => {
    render(SettingsForm, { props: { ...defaultProps, pauseMediaOnBreak: false } });
    const input = getToggleByLabel('Auto-pause');
    expect(input.checked).toBe(false);
  });

  it('3-9e: onPauseMediaChange is called when toggle changes', async () => {
    const onPauseMediaChange = vi.fn();
    render(SettingsForm, { props: { ...defaultProps, onPauseMediaChange } });
    const input = getToggleByLabel('Auto-pause');
    await fireEvent.click(input);
    expect(onPauseMediaChange).toHaveBeenCalledTimes(1);
  });
});
