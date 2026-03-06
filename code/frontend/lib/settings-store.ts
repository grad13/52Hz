// meta: checked=2026-03-07
import { load, type Store } from "@tauri-apps/plugin-store";
import type { TimerSettings } from "./timer";

// ---------------------------------------------------------------
// Store helper
// ---------------------------------------------------------------

function getStore(): Promise<Store> {
  return load("settings.json", { autoSave: true } as Parameters<typeof load>[1]);
}

// ---------------------------------------------------------------
// Type definitions
// ---------------------------------------------------------------

export interface DisplaySettings {
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  shortBreaksBeforeLong: number;
}

export type PresencePosition = "top-right" | "top-left" | "bottom-right" | "bottom-left";

// ---------------------------------------------------------------
// Unit conversion (DisplaySettings ↔ TimerSettings)
// ---------------------------------------------------------------

export function toTimerSettings(d: DisplaySettings): TimerSettings {
  return {
    focus_duration_secs: d.focusMinutes * 60,
    short_break_duration_secs: d.shortBreakMinutes * 60,
    long_break_duration_secs: d.longBreakMinutes * 60,
    short_breaks_before_long: d.shortBreaksBeforeLong,
  };
}

export function toDisplaySettings(s: TimerSettings): DisplaySettings {
  return {
    focusMinutes: s.focus_duration_secs / 60,
    shortBreakMinutes: s.short_break_duration_secs / 60,
    longBreakMinutes: s.long_break_duration_secs / 60,
    shortBreaksBeforeLong: s.short_breaks_before_long,
  };
}

// ---------------------------------------------------------------
// Timer settings (load / save)
// ---------------------------------------------------------------

export async function loadSettings(): Promise<DisplaySettings | null> {
  try {
    const store = await getStore();
    const fm = await store.get<number>("focus_minutes");
    const sbm = await store.get<number>("short_break_minutes");
    const lbm = await store.get<number>("long_break_minutes");
    const sbbl = await store.get<number>("short_breaks_before_long");

    if (fm == null && sbm == null && lbm == null && sbbl == null) {
      return null;
    }

    return {
      focusMinutes: fm ?? 20,
      shortBreakMinutes: sbm ?? 1,
      longBreakMinutes: lbm ?? 3,
      shortBreaksBeforeLong: sbbl ?? 3,
    };
  } catch {
    return null;
  }
}

export async function saveSettings(d: DisplaySettings): Promise<void> {
  const store = await getStore();
  await store.set("focus_minutes", d.focusMinutes);
  await store.set("short_break_minutes", d.shortBreakMinutes);
  await store.set("long_break_minutes", d.longBreakMinutes);
  await store.set("short_breaks_before_long", d.shortBreaksBeforeLong);
}

// ---------------------------------------------------------------
// Individual settings (load / save)
// ---------------------------------------------------------------

export async function loadPauseMediaOnBreak(): Promise<boolean> {
  try {
    const store = await getStore();
    const val = await store.get<boolean>("pause_media_on_break");
    return val ?? false;
  } catch {
    return false;
  }
}

export async function savePauseMediaOnBreak(enabled: boolean): Promise<void> {
  const store = await getStore();
  await store.set("pause_media_on_break", enabled);
}

export async function loadHideTrayIcon(): Promise<boolean> {
  try {
    const store = await getStore();
    const val = await store.get<boolean>("hide_tray_icon");
    return val ?? false;
  } catch {
    return false;
  }
}

export async function saveHideTrayIcon(enabled: boolean): Promise<void> {
  const store = await getStore();
  await store.set("hide_tray_icon", enabled);
}

export async function loadTickVolume(): Promise<number> {
  try {
    const store = await getStore();
    const val = await store.get<number | boolean>("tick_sound");
    if (val === true) return 0.5;
    if (val === false || val == null) return 0;
    return val;
  } catch {
    return 0;
  }
}

export async function saveTickVolume(volume: number): Promise<void> {
  const store = await getStore();
  await store.set("tick_sound", volume);
}

export async function loadPresenceToast(): Promise<boolean> {
  try {
    const store = await getStore();
    const val = await store.get<boolean>("presence_toast");
    return val ?? true; // default ON
  } catch {
    return true;
  }
}

export async function savePresenceToast(enabled: boolean): Promise<void> {
  const store = await getStore();
  await store.set("presence_toast", enabled);
}

export async function loadPresencePosition(): Promise<PresencePosition> {
  try {
    const store = await getStore();
    const val = await store.get<PresencePosition>("presence_position");
    return val ?? "top-right";
  } catch {
    return "top-right";
  }
}

export async function savePresencePosition(pos: PresencePosition): Promise<void> {
  const store = await getStore();
  await store.set("presence_position", pos);
}

