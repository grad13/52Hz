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
  it('3-1: 4 input fields are displayed', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByLabelText('Focus')).toBeTruthy();
    expect(screen.getByLabelText('Short Break')).toBeTruthy();
    expect(screen.getByLabelText('Long Break')).toBeTruthy();
    expect(screen.getByLabelText('Cycles')).toBeTruthy();
  });

  it('3-4: Focus time field has min=1, max=120', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('focus') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('120');
  });

  it('3-5: Short break field has min=1, no max attribute', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('short-break') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('');
  });

  it('3-6: Long break field has min=1, max=30', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('long-break') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('30');
  });

  it('3-7: Cycles field has min=1, max=10', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    const input = document.getElementById('cycles') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.type).toBe('number');
    expect(input.min).toBe('1');
    expect(input.max).toBe('10');
  });

  it('3-8: Hide icon toggle is displayed', () => {
    render(SettingsForm, { props: { ...defaultProps } });
    expect(screen.getByText('Hide icon')).toBeTruthy();
  });

  it('3-8b: Hide icon toggle is checked when hideTrayIcon=true', () => {
    const { container } = render(SettingsForm, { props: { ...defaultProps, hideTrayIcon: true } });
    const label = screen.getByText('Hide icon');
    const row = label.closest('.toggle-row');
    const input = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
    expect(input).toBeTruthy();
    expect(input.checked).toBe(true);
  });

  it('3-8c: onHideTrayIconChange is called when hide icon toggle changes', async () => {
    const onHideTrayIconChange = vi.fn();
    const { container } = render(SettingsForm, { props: { ...defaultProps, onHideTrayIconChange } });
    const label = screen.getByText('Hide icon');
    const row = label.closest('.toggle-row');
    const input = row?.querySelector('input[type="checkbox"]') as HTMLInputElement;
    await fireEvent.click(input);
    expect(onHideTrayIconChange).toHaveBeenCalledTimes(1);
  });
});
