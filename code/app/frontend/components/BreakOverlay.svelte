<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    type TimerState,
    type TimerPhase,
    remainingSecs,
    formatTime,
    getTimerState,
    skipBreak,
    onTimerTick,
    onBreakEnd,
  } from "../lib/timer";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { _ } from "svelte-i18n";

  let remaining = $state("--:--");
  let phase: TimerPhase = $state("ShortBreak");
  let unlistenTick: (() => void) | null = null;
  let unlistenEnd: (() => void) | null = null;
  let initialized = $state(false);

  function handleTick(state: TimerState) {
    remaining = formatTime(remainingSecs(state));
    phase = state.phase;
    initialized = true;
  }

  async function handleSkip() {
    await skipBreak();
  }

  async function handleBreakEnd() {
    const win = getCurrentWindow();
    await win.close();
  }

  onMount(async () => {
    // Get initial state immediately so we don't show stale defaults
    const state = await getTimerState();
    handleTick(state);

    unlistenTick = (await onTimerTick(handleTick)) as unknown as () => void;
    unlistenEnd = (await onBreakEnd(handleBreakEnd)) as unknown as () => void;
  });

  onDestroy(() => {
    unlistenTick?.();
    unlistenEnd?.();
  });
</script>

<div class="overlay" class:visible={initialized}>
  <div class="content">
    <h1>{phase === "LongBreak" ? $_("break.long_title") : phase === "ShortBreak" ? $_("break.short_title") : $_("break.fallback_title")}</h1>
    <p class="subtitle">{phase === "LongBreak" ? $_("break.long_subtitle") : phase === "ShortBreak" ? $_("break.short_subtitle") : ""}</p>

    <div class="timer-ring">
      <div class="timer">{remaining}</div>
    </div>

    <button class="skip-btn" onclick={handleSkip}>{$_("break.skip")}</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: radial-gradient(ellipse at center, #12121a 0%, #0a0a0e 70%);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: default;
    overflow: hidden;
  }

  .content {
    text-align: center;
    color: #fff;
    display: flex;
    flex-direction: column;
    align-items: center;
    opacity: 0;
  }

  .overlay.visible .content {
    animation: fadeIn 0.8s ease-out forwards;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.94) translateY(8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  h1 {
    font-size: 1.6rem;
    font-weight: 300;
    margin-bottom: 0.5rem;
    letter-spacing: 0.1em;
    color: rgba(255, 255, 255, 0.85);
  }

  .subtitle {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.35);
    margin-bottom: 2.5rem;
    font-weight: 300;
    letter-spacing: 0.04em;
  }

  .timer-ring {
    width: 160px;
    height: 160px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 2.5rem;
    animation: breathe 4s ease-in-out infinite;
    box-shadow: 0 0 40px rgba(108, 92, 231, 0.08),
                inset 0 0 40px rgba(108, 92, 231, 0.03);
  }

  @keyframes breathe {
    0%, 100% {
      border-color: rgba(255, 255, 255, 0.1);
      box-shadow: 0 0 40px rgba(108, 92, 231, 0.05),
                  inset 0 0 40px rgba(108, 92, 231, 0.02);
    }
    50% {
      border-color: rgba(255, 255, 255, 0.2);
      box-shadow: 0 0 60px rgba(108, 92, 231, 0.12),
                  inset 0 0 40px rgba(108, 92, 231, 0.05);
    }
  }

  .timer {
    font-size: 2.8rem;
    font-weight: 200;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.1em;
    color: rgba(255, 255, 255, 0.8);
  }

  .skip-btn {
    padding: 0.6rem 2.5rem;
    font-size: 0.8rem;
    font-weight: 400;
    letter-spacing: 0.08em;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-pill);
    background: transparent;
    color: rgba(255, 255, 255, 0.4);
    cursor: pointer;
    transition: all 0.4s var(--ease-out);
  }

  .skip-btn:hover {
    border-color: rgba(255, 255, 255, 0.3);
    color: rgba(255, 255, 255, 0.7);
    background: rgba(255, 255, 255, 0.04);
  }
</style>
