import { load } from "@tauri-apps/plugin-store";
import type { TimerSettings } from "./timer";

export interface DisplaySettings {
  focusMinutes: number;
  shortBreakSecs: number;
  longBreakMinutes: number;
  shortBreaksBeforeLong: number;
}

export function toTimerSettings(d: DisplaySettings): TimerSettings {
  return {
    focus_duration_secs: d.focusMinutes * 60,
    short_break_duration_secs: d.shortBreakSecs,
    long_break_duration_secs: d.longBreakMinutes * 60,
    short_breaks_before_long: d.shortBreaksBeforeLong,
  };
}

export function toDisplaySettings(s: TimerSettings): DisplaySettings {
  return {
    focusMinutes: s.focus_duration_secs / 60,
    shortBreakSecs: s.short_break_duration_secs,
    longBreakMinutes: s.long_break_duration_secs / 60,
    shortBreaksBeforeLong: s.short_breaks_before_long,
  };
}

export async function loadSettings(): Promise<DisplaySettings | null> {
  try {
    const store = await load("settings.json", { autoSave: true } as Parameters<typeof load>[1]);
    const fm = await store.get<number>("focus_minutes");
    const sbs = await store.get<number>("short_break_secs");
    const lbm = await store.get<number>("long_break_minutes");
    const sbbl = await store.get<number>("short_breaks_before_long");

    if (fm == null && sbs == null && lbm == null && sbbl == null) {
      return null;
    }

    return {
      focusMinutes: fm ?? 20,
      shortBreakSecs: sbs ?? 20,
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
  await store.set("short_break_secs", d.shortBreakSecs);
  await store.set("long_break_minutes", d.longBreakMinutes);
  await store.set("short_breaks_before_long", d.shortBreaksBeforeLong);
}
