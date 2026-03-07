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
    resetTimer,
    quitApp,
    getTodaySessions,
    setTrayIconVisible,
  } from "../lib/timer";
  import {
    loadSettings,
    saveSettings,
    toTimerSettings,
    loadPauseMediaOnBreak,
    savePauseMediaOnBreak,
    loadHideTrayIcon,
    saveHideTrayIcon,
    loadTickVolume,
    saveTickVolume,
    loadPresenceToast,
    savePresenceToast,
    loadPresencePosition,
    savePresencePosition,
    loadPresenceLevel,
    savePresenceLevel,
    type PresencePosition,
    type PresenceLevel,
  } from "../lib/settings-store";
  import { emit, emitTo } from "@tauri-apps/api/event";
  import {
    enable as enableAutostart,
    disable as disableAutostart,
    isEnabled as isAutostartEnabled,
  } from "@tauri-apps/plugin-autostart";
  import TimerStatus from "./TimerStatus.svelte";
  import SettingsForm from "./SettingsForm.svelte";
  import tickSrc from "../assets/tick.mp3";

  let timerState: TimerState | null = $state(null);
  let remaining = $state("--:--");
  let paused = $state(false);

  let cycleCompleted = $derived.by(() => {
    if (!timerState) return 0;
    return timerState.short_break_count;
  });
  let cycleTotal = $derived.by(() => {
    if (!timerState) return 1;
    return timerState.settings.short_breaks_before_long;
  });
  let isLongBreak = $derived.by(() => {
    if (!timerState) return false;
    return timerState.phase === "LongBreak";
  });

  // Settings form values (in minutes/seconds for display)
  let focusMinutes = $state(20);
  let shortBreakMinutes = $state(1);
  let longBreakMinutes = $state(3);
  let shortBreaksBeforeLong = $state(3);

  let settingsLoaded = $state(false);
  let autostartEnabled = $state(false);
  let pauseMediaOnBreak = $state(false);
  let hideTrayIcon = $state(false);
  let tickVolume = $state(0);
  let presenceToast = $state(true);
  let presencePosition: PresencePosition = $state("top-right");
  let presenceLevel: PresenceLevel = $state("dynamic");
  let todaySessions = $state(0);
  let tickAudio: HTMLAudioElement | null = null;

  let unlistenTick: (() => void) | null = null;
  let unlistenPhaseChanged: (() => void) | null = null;
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function handleTick(state: TimerState) {
    timerState = state;
    remaining = formatTime(remainingSecs(state));
    paused = state.paused;
    if (tickVolume > 0 && !state.paused && tickAudio) {
      tickAudio.volume = tickVolume;
      tickAudio.play().catch(() => {});
    }
  }

  async function handleTogglePause() {
    paused = await togglePause();
  }

  async function handleSaveSettings() {
    const display = { focusMinutes, shortBreakMinutes, longBreakMinutes, shortBreaksBeforeLong };
    await updateSettings(toTimerSettings(display));
    await saveSettings(display);
  }

  async function handlePauseMediaChange(enabled: boolean) {
    pauseMediaOnBreak = enabled;
    await savePauseMediaOnBreak(enabled);
  }

  async function handleTickVolumeChange(volume: number) {
    tickVolume = volume;
    await saveTickVolume(volume);
  }

  async function handlePresenceToastChange(enabled: boolean) {
    presenceToast = enabled;
    await savePresenceToast(enabled);
    await emitTo("presence-toast", "presence-toast-toggle", enabled);
  }

  async function handlePresencePositionChange(pos: PresencePosition) {
    presencePosition = pos;
    await savePresencePosition(pos);
    await emitTo("presence-toast", "presence-position-change", pos);
    await emit("presence-position-change", pos);
  }

  async function handlePresenceLevelChange(level: PresenceLevel) {
    presenceLevel = level;
    await savePresenceLevel(level);
    await emitTo("presence-toast", "presence-level-setting", level);
    await emit("presence-level-change", level);
  }

  async function handleHideTrayIconChange(enabled: boolean) {
    hideTrayIcon = enabled;
    await saveHideTrayIcon(enabled);
    await setTrayIconVisible(!enabled);
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
    settingsLoaded = true;
  }

  $effect(() => {
    // Subscribe to all 4 settings values
    const _deps = [focusMinutes, shortBreakMinutes, longBreakMinutes, shortBreaksBeforeLong];
    void _deps;
    if (!settingsLoaded) return;
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => handleSaveSettings(), 500);
  });

  onMount(async () => {
    await loadSavedSettings();
    autostartEnabled = await isAutostartEnabled().catch(() => false);
    pauseMediaOnBreak = await loadPauseMediaOnBreak();
    hideTrayIcon = await loadHideTrayIcon();
    tickVolume = await loadTickVolume();
    presenceToast = await loadPresenceToast();
    presencePosition = await loadPresencePosition();
    presenceLevel = await loadPresenceLevel();
    tickAudio = new Audio(tickSrc);
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
  <TimerStatus {remaining} {paused} {cycleCompleted} {cycleTotal} {isLongBreak} {todaySessions} onTogglePause={handleTogglePause} />

  <div class="divider"></div>

  <SettingsForm
    bind:focusMinutes
    bind:shortBreakMinutes
    bind:longBreakMinutes
    bind:shortBreaksBeforeLong
    {autostartEnabled}
    onAutostartChange={handleAutostartChange}
    {pauseMediaOnBreak}
    onPauseMediaChange={handlePauseMediaChange}
    {hideTrayIcon}
    onHideTrayIconChange={handleHideTrayIconChange}
    {tickVolume}
    onTickVolumeChange={handleTickVolumeChange}
    {presenceToast}
    onPresenceToastChange={handlePresenceToastChange}
    {presencePosition}
    onPresencePositionChange={handlePresencePositionChange}
    {presenceLevel}
    onPresenceLevelChange={handlePresenceLevelChange}
  />

  <div class="bottom-row">
    <button class="stop-link" onclick={resetTimer}>■ 停止</button>
    <span class="sep">|</span>
    <button class="quit-link" onclick={quitApp}>アプリを終了</button>
  </div>
</div>

<style>
  .tray-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 0.75rem;
    gap: 0.5rem;
    background: var(--bg);
    border-radius: 10px;
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 0.1rem 0;
  }

  .bottom-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    margin-top: auto;
  }

  .bottom-row .stop-link {
    font-size: 0.75rem;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color var(--duration-normal) var(--ease-out);
  }

  .bottom-row .stop-link:hover {
    color: var(--danger);
  }

  .bottom-row .sep {
    color: var(--border);
    font-size: 0.65rem;
  }

  .bottom-row .quit-link {
    font-size: 0.75rem;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color var(--duration-normal) var(--ease-out);
  }

  .bottom-row .quit-link:hover {
    color: var(--danger);
  }
</style>
