import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface TimerSettings {
  focus_duration_secs: number;
  short_break_duration_secs: number;
  long_break_duration_secs: number;
  short_breaks_before_long: number;
}

export type TimerPhase = "Focus" | "ShortBreak" | "LongBreak";

export interface TimerState {
  phase: TimerPhase;
  paused: boolean;
  elapsed_secs: number;
  phase_duration_secs: number;
  short_break_count: number;
  settings: TimerSettings;
}

export function remainingSecs(state: TimerState): number {
  return Math.max(0, state.phase_duration_secs - state.elapsed_secs);
}

export function formatTime(totalSecs: number): string {
  const mins = Math.floor(totalSecs / 60);
  const secs = totalSecs % 60;
  return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
}

export async function getTimerState(): Promise<TimerState> {
  return invoke("get_timer_state");
}

export async function pauseTimer(): Promise<void> {
  return invoke("pause_timer");
}

export async function resumeTimer(): Promise<void> {
  return invoke("resume_timer");
}

export async function togglePause(): Promise<boolean> {
  return invoke("toggle_pause");
}

export async function skipBreak(): Promise<void> {
  return invoke("skip_break");
}

export async function updateSettings(settings: TimerSettings): Promise<void> {
  return invoke("update_settings", { settings });
}

export async function closeBreakOverlay(): Promise<void> {
  return invoke("close_break_overlay");
}

export async function resetTimer(): Promise<void> {
  return invoke("reset_timer");
}

export async function quitApp(): Promise<void> {
  return invoke("quit_app");
}

export function onTimerTick(callback: (state: TimerState) => void) {
  return listen<TimerState>("timer-tick", (event) => callback(event.payload));
}

export function onPhaseChanged(callback: (state: TimerState) => void) {
  return listen<TimerState>("phase-changed", (event) =>
    callback(event.payload)
  );
}

export function onBreakStart(callback: (state: TimerState) => void) {
  return listen<TimerState>("break-start", (event) =>
    callback(event.payload)
  );
}

export function onBreakEnd(callback: () => void) {
  return listen("break-end", () => callback());
}

export async function acceptBreak(): Promise<void> {
  return invoke("accept_break");
}

export async function extendFocus(secs: number): Promise<void> {
  return invoke("extend_focus", { secs });
}

export async function skipBreakFromFocus(): Promise<void> {
  return invoke("skip_break_from_focus");
}

export async function getTodaySessions(): Promise<number> {
  return invoke("get_today_sessions");
}

export async function setTrayIconVisible(visible: boolean): Promise<void> {
  return invoke("set_tray_icon_visible", { visible });
}

export interface CassetteInfo {
  path: string;
  title: string;
}

export async function listCassettes(): Promise<CassetteInfo[]> {
  return invoke("list_cassettes");
}

export async function switchCassette(path: string): Promise<void> {
  return invoke("switch_cassette", { path });
}

export async function openCassetteFolder(): Promise<void> {
  return invoke("open_cassette_folder");
}
