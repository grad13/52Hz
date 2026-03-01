<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    type TimerState,
    remainingSecs,
    formatTime,
    getTimerState,
    togglePause,
    updateSettings,
    onTimerTick,
    onPhaseChanged,
    quitApp,
    getTodaySessions,
  } from "../lib/timer";
  import {
    loadSettings,
    saveSettings,
    toTimerSettings,
  } from "../lib/settings-store";
  import {
    enable as enableAutostart,
    disable as disableAutostart,
    isEnabled as isAutostartEnabled,
  } from "@tauri-apps/plugin-autostart";
  import TimerStatus from "./TimerStatus.svelte";
  import TimerControls from "./TimerControls.svelte";
  import SettingsForm from "./SettingsForm.svelte";

  let timerState: TimerState | null = $state(null);
  let remaining = $state("--:--");
  let phaseLabel = $state("Focus");
  let paused = $state(false);

  // Settings form values (in minutes/seconds for display)
  let focusMinutes = $state(20);
  let shortBreakMinutes = $state(1);
  let longBreakMinutes = $state(3);
  let shortBreaksBeforeLong = $state(3);

  let autostartEnabled = $state(false);
  let todaySessions = $state(0);

  let unlistenTick: (() => void) | null = null;
  let unlistenPhaseChanged: (() => void) | null = null;

  const phaseLabels: Record<string, string> = {
    Focus: "フォーカス中",
    ShortBreak: "短い休憩中",
    LongBreak: "長い休憩中",
  };

  function handleTick(state: TimerState) {
    timerState = state;
    remaining = formatTime(remainingSecs(state));
    phaseLabel = phaseLabels[state.phase] || state.phase;
    paused = state.paused;
  }

  async function handleTogglePause() {
    paused = await togglePause();
  }

  async function handleSaveSettings() {
    const display = { focusMinutes, shortBreakMinutes, longBreakMinutes, shortBreaksBeforeLong };
    await updateSettings(toTimerSettings(display));
    await saveSettings(display);
  }

  async function handleAutostartChange(enabled: boolean) {
    try {
      if (enabled) {
        await enableAutostart();
      } else {
        await disableAutostart();
      }
      autostartEnabled = await isAutostartEnabled();
    } catch {
      autostartEnabled = await isAutostartEnabled().catch(() => false);
    }
  }

  async function loadSavedSettings() {
    const saved = await loadSettings();
    if (saved) {
      focusMinutes = saved.focusMinutes;
      shortBreakMinutes = saved.shortBreakMinutes;
      longBreakMinutes = saved.longBreakMinutes;
      shortBreaksBeforeLong = saved.shortBreaksBeforeLong;
      await updateSettings(toTimerSettings(saved));
    }
  }

  onMount(async () => {
    await loadSavedSettings();
    autostartEnabled = await isAutostartEnabled().catch(() => false);
    todaySessions = await getTodaySessions();
    const state = await getTimerState();
    handleTick(state);
    unlistenTick = (await onTimerTick(handleTick)) as unknown as () => void;
    unlistenPhaseChanged = (await onPhaseChanged(async () => {
      todaySessions = await getTodaySessions();
    })) as unknown as () => void;
  });

  onDestroy(() => {
    unlistenTick?.();
    unlistenPhaseChanged?.();
  });
</script>

<div class="tray-panel">
  <header>
    <h2>52Hz</h2>
  </header>

  <TimerStatus {phaseLabel} {remaining} {paused} />
  <div class="session-count">今日のセッション: {todaySessions} 回</div>
  <TimerControls {paused} onTogglePause={handleTogglePause} onQuit={quitApp} />
  <SettingsForm
    bind:focusMinutes
    bind:shortBreakMinutes
    bind:longBreakMinutes
    bind:shortBreaksBeforeLong
    {autostartEnabled}
    onSave={handleSaveSettings}
    onAutostartChange={handleAutostartChange}
  />
</div>

<style>
  .tray-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1rem;
    gap: 0.8rem;
  }

  header h2 {
    font-size: 1rem;
    font-weight: 600;
    text-align: center;
    color: var(--text-secondary);
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .session-count {
    font-size: 0.8rem;
    text-align: center;
    color: var(--text-secondary);
  }
</style>
