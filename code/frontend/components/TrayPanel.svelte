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
    listCassettes,
    switchCassette,
    openCassetteFolder,
    type CassetteInfo,
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
    loadPresenceMaxToasts,
    savePresenceMaxToasts,
    loadPresenceShowIcon,
    savePresenceShowIcon,
    loadPresenceLikeIcon,
    savePresenceLikeIcon,
    type PresencePosition,
    type PresenceLevel,
    type PresenceLikeIcon,
  } from "../lib/settings-store";
  import { emit, emitTo } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import {
    enable as enableAutostart,
    disable as disableAutostart,
    isEnabled as isAutostartEnabled,
  } from "@tauri-apps/plugin-autostart";
  import TimerStatus from "./TimerStatus.svelte";
  import SettingsForm from "./SettingsForm.svelte";
  import { tick } from "svelte";
  import tickSrc from "../assets/tick.mp3";
  import { _, locale } from "svelte-i18n";
  import { saveLocale, type AppLocale } from "../lib/settings-store";

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
  let presenceMaxToasts = $state(4);
  let presenceShowIcon = $state(true);
  let presenceLikeIcon: PresenceLikeIcon = $state("heart");
  let cassettes: CassetteInfo[] = $state([]);
  let currentCassette = $state("");
  let todaySessions = $state(0);
  let tickAudio: HTMLAudioElement | null = null;
  let panelEl: HTMLDivElement;
  const win = getCurrentWindow();

  async function handleLocaleChange(loc: AppLocale) {
    locale.set(loc);
    await saveLocale(loc);
    await syncPanelHeight();
  }

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

  async function syncPanelHeight() {
    await tick();
    if (panelEl) {
      const h = panelEl.scrollHeight;
      await win.setSize(new LogicalSize(320, h));
    }
  }

  async function handlePresenceToastChange(enabled: boolean) {
    presenceToast = enabled;
    await savePresenceToast(enabled);
    await emitTo("presence-toast", "presence-toast-toggle", enabled);
    await syncPanelHeight();
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

  async function handlePresenceMaxToastsChange(n: number) {
    presenceMaxToasts = n;
    await savePresenceMaxToasts(n);
    await emitTo("presence-toast", "presence-max-toasts-change", n);
  }

  async function handlePresenceShowIconChange(v: boolean) {
    presenceShowIcon = v;
    await savePresenceShowIcon(v);
    await emitTo("presence-toast", "presence-show-icon-change", v);
  }

  async function handlePresenceLikeIconChange(v: PresenceLikeIcon) {
    presenceLikeIcon = v;
    await savePresenceLikeIcon(v);
    await emitTo("presence-toast", "presence-like-icon-change", v);
  }

  async function handleCassetteChange(path: string) {
    currentCassette = path;
    await switchCassette(path);
  }

  async function handleOpenCassetteFolder() {
    await openCassetteFolder();
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
    presenceMaxToasts = await loadPresenceMaxToasts();
    presenceShowIcon = await loadPresenceShowIcon();
    presenceLikeIcon = await loadPresenceLikeIcon();
    cassettes = await listCassettes().catch(() => []);
    if (cassettes.length > 0) currentCassette = cassettes[0].path;
    tickAudio = new Audio(tickSrc);
    todaySessions = await getTodaySessions();
    const state = await getTimerState();
    handleTick(state);
    unlistenTick = (await onTimerTick(handleTick)) as unknown as () => void;
    unlistenPhaseChanged = (await onPhaseChanged(async () => {
      todaySessions = await getTodaySessions();
    })) as unknown as () => void;
    await syncPanelHeight();
  });

  onDestroy(() => {
    unlistenTick?.();
    unlistenPhaseChanged?.();
  });
</script>

<div class="tray-panel" bind:this={panelEl}>
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
    {presenceMaxToasts}
    onPresenceMaxToastsChange={handlePresenceMaxToastsChange}
    {presenceShowIcon}
    onPresenceShowIconChange={handlePresenceShowIconChange}
    {presenceLikeIcon}
    onPresenceLikeIconChange={handlePresenceLikeIconChange}
    {cassettes}
    {currentCassette}
    onCassetteChange={handleCassetteChange}
    onOpenCassetteFolder={handleOpenCassetteFolder}
  />

  <div class="lang-row">
    <span class="lang-label">{$_("tray.language")}</span>
    <div class="lang-toggle">
      <button class="lang-btn" class:active={$locale === "en"} onclick={() => handleLocaleChange("en")}>EN</button>
      <button class="lang-btn" class:active={$locale === "ja"} onclick={() => handleLocaleChange("ja")}>JA</button>
    </div>
  </div>

  <div class="bottom-row">
    <button class="stop-link" onclick={resetTimer}>{$_("tray.stop")}</button>
    <span class="sep">|</span>
    <button class="quit-link" onclick={quitApp}>{$_("tray.quit")}</button>
  </div>
</div>

<style>
  .tray-panel {
    display: flex;
    flex-direction: column;
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

  .lang-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.2rem 0;
  }

  .lang-label {
    font-size: 0.7rem;
    color: var(--text-tertiary);
  }

  .lang-toggle {
    display: flex;
    gap: 1px;
  }

  .lang-btn {
    padding: 1px 6px;
    font-size: 0.6rem;
    font-weight: 600;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .lang-btn:first-child {
    border-radius: 3px 0 0 3px;
  }

  .lang-btn:last-child {
    border-radius: 0 3px 3px 0;
    border-left: none;
  }

  .lang-btn.active {
    background: var(--success);
    color: #1a1a2e;
    border-color: var(--success);
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
