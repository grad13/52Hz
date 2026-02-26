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
    quitApp,
  } from "../lib/timer";
  import {
    loadSettings,
    saveSettings,
    toTimerSettings,
  } from "../lib/settings-store";
  import TimerStatus from "./TimerStatus.svelte";
  import TimerControls from "./TimerControls.svelte";
  import SettingsForm from "./SettingsForm.svelte";

  let timerState: TimerState | null = $state(null);
  let remaining = $state("--:--");
  let phaseLabel = $state("Focus");
  let paused = $state(false);

  // Settings form values (in minutes/seconds for display)
  let focusMinutes = $state(20);
  let shortBreakSecs = $state(20);
  let longBreakMinutes = $state(3);
  let shortBreaksBeforeLong = $state(3);

  let unlistenTick: (() => void) | null = null;

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
    const display = { focusMinutes, shortBreakSecs, longBreakMinutes, shortBreaksBeforeLong };
    await updateSettings(toTimerSettings(display));
    await saveSettings(display);
  }

  async function loadSavedSettings() {
    const saved = await loadSettings();
    if (saved) {
      focusMinutes = saved.focusMinutes;
      shortBreakSecs = saved.shortBreakSecs;
      longBreakMinutes = saved.longBreakMinutes;
      shortBreaksBeforeLong = saved.shortBreaksBeforeLong;
      await updateSettings(toTimerSettings(saved));
    }
  }

  onMount(async () => {
    await loadSavedSettings();
    const state = await getTimerState();
    handleTick(state);
    unlistenTick = (await onTimerTick(handleTick)) as unknown as () => void;
  });

  onDestroy(() => {
    unlistenTick?.();
  });
</script>

<div class="tray-panel">
  <header>
    <h2>RestRun</h2>
  </header>

  <TimerStatus {phaseLabel} {remaining} {paused} />
  <TimerControls {paused} onTogglePause={handleTogglePause} onQuit={quitApp} />
  <SettingsForm
    bind:focusMinutes
    bind:shortBreakSecs
    bind:longBreakMinutes
    bind:shortBreaksBeforeLong
    onSave={handleSaveSettings}
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
</style>
