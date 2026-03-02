import { load } from "@tauri-apps/plugin-store";
import type { TimerSettings } from "./timer";

export interface DisplaySettings {
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  shortBreaksBeforeLong: number;
}

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

export async function loadSettings(): Promise<DisplaySettings | null> {
  try {
    const store = await load("settings.json", { autoSave: true } as Parameters<typeof load>[1]);
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
  const store = await load("settings.json", { autoSave: true } as Parameters<typeof load>[1]);
  await store.set("focus_minutes", d.focusMinutes);
  await store.set("short_break_minutes", d.shortBreakMinutes);
  await store.set("long_break_minutes", d.longBreakMinutes);
  await store.set("short_breaks_before_long", d.shortBreaksBeforeLong);
}

export async function loadPauseMediaOnBreak(): Promise<boolean> {
  try {
    const store = await load("settings.json", { autoSave: true } as Parameters<typeof load>[1]);
    const val = await store.get<boolean>("pause_media_on_break");
    return val ?? false;
  } catch {
    return false;
  }
}

export async function savePauseMediaOnBreak(enabled: boolean): Promise<void> {
  const store = await load("settings.json", { autoSave: true } as Parameters<typeof load>[1]);
  await store.set("pause_media_on_break", enabled);
}
