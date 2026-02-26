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

<div class="settings">
  <header>
    <h2>RestRun</h2>
  </header>

  <section class="status">
    <div class="phase">{phaseLabel}</div>
    <div class="time">{remaining}</div>
    {#if paused}
      <div class="paused-badge">一時停止中</div>
    {/if}
  </section>

  <section class="controls">
    <button class="control-btn" onclick={handleTogglePause}>
      {paused ? "▶ 再開" : "⏸ 一時停止"}
    </button>
    <button class="control-btn quit-btn" onclick={quitApp}>
      ✕ 終了
    </button>
  </section>

  <section class="form">
    <div class="field">
      <label for="focus">フォーカス時間</label>
      <div class="input-group">
        <input
          id="focus"
          type="number"
          min="1"
          max="120"
          bind:value={focusMinutes}
        />
        <span class="unit">分</span>
      </div>
    </div>

    <div class="field">
      <label for="short-break">短い休憩</label>
      <div class="input-group">
        <input
          id="short-break"
          type="number"
          min="5"
          max="300"
          bind:value={shortBreakSecs}
        />
        <span class="unit">秒</span>
      </div>
    </div>

    <div class="field">
      <label for="long-break">長い休憩</label>
      <div class="input-group">
        <input
          id="long-break"
          type="number"
          min="1"
          max="30"
          bind:value={longBreakMinutes}
        />
        <span class="unit">分</span>
      </div>
    </div>

    <div class="field">
      <label for="cycles">長い休憩までの回数</label>
      <div class="input-group">
        <input
          id="cycles"
          type="number"
          min="1"
          max="10"
          bind:value={shortBreaksBeforeLong}
        />
        <span class="unit">回</span>
      </div>
    </div>

    <button class="save-btn" onclick={handleSaveSettings}>設定を保存</button>
  </section>
</div>

<style>
  .settings {
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

  .status {
    text-align: center;
    padding: 0.8rem 0;
  }

  .phase {
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin-bottom: 0.3rem;
  }

  .time {
    font-size: 2.8rem;
    font-weight: 300;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.05em;
  }

  .paused-badge {
    display: inline-block;
    margin-top: 0.4rem;
    padding: 0.15rem 0.6rem;
    font-size: 0.75rem;
    background: var(--danger);
    border-radius: 4px;
    color: #fff;
  }

  .controls {
    display: flex;
    justify-content: center;
  }

  .control-btn {
    padding: 0.5rem 1.5rem;
    font-size: 0.9rem;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    background: var(--accent);
    color: var(--text);
    cursor: pointer;
    transition: background 0.2s;
  }

  .control-btn:hover {
    background: var(--accent-light);
  }

  .quit-btn {
    background: transparent;
    border-color: var(--danger);
    color: var(--danger);
  }

  .quit-btn:hover {
    background: var(--danger);
    color: #fff;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    flex: 1;
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .field label {
    font-size: 0.85rem;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .input-group {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .input-group input {
    width: 60px;
    padding: 0.3rem 0.5rem;
    font-size: 0.9rem;
    text-align: right;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text);
  }

  .input-group input:focus {
    outline: none;
    border-color: var(--accent-light);
  }

  .unit {
    font-size: 0.8rem;
    color: var(--text-secondary);
    width: 1.5em;
  }

  .save-btn {
    margin-top: auto;
    padding: 0.5rem;
    font-size: 0.9rem;
    border: none;
    border-radius: 6px;
    background: var(--success);
    color: #1a1a2e;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .save-btn:hover {
    opacity: 0.9;
  }
</style>
