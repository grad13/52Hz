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

  let remaining = $state("--:--");
  let phase: TimerPhase = $state("ShortBreak");
  let unlistenTick: (() => void) | null = null;
  let unlistenEnd: (() => void) | null = null;
  let initialized = $state(false);

  const messages: Record<TimerPhase, { title: string; subtitle: string }> = {
    ShortBreak: {
      title: "目を休めましょう",
      subtitle: "遠くを見て、まばたきをしましょう",
    },
    LongBreak: {
      title: "立ち上がってストレッチ",
      subtitle: "体を動かして、深呼吸しましょう",
    },
    Focus: {
      title: "",
      subtitle: "",
    },
  };

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
    <h1>{messages[phase]?.title || "休憩中"}</h1>
    <p class="subtitle">{messages[phase]?.subtitle || ""}</p>

    <div class="timer-ring">
      <div class="timer">{remaining}</div>
    </div>

    <button class="skip-btn" onclick={handleSkip}>スキップ</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: #0a0a0e;
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
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  h1 {
    font-size: 2rem;
    font-weight: 300;
    margin-bottom: 0.5rem;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.9);
  }

  .subtitle {
    font-size: 1rem;
    color: rgba(255, 255, 255, 0.45);
    margin-bottom: 3rem;
    font-weight: 300;
  }

  .timer-ring {
    width: 200px;
    height: 200px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.15);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 3rem;
  }

  .timer {
    font-size: 3.5rem;
    font-weight: 200;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.85);
  }

  .skip-btn {
    padding: 0.6rem 2.5rem;
    font-size: 0.85rem;
    font-weight: 400;
    letter-spacing: 0.05em;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 20px;
    background: transparent;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    transition: all 0.3s;
  }

  .skip-btn:hover {
    border-color: rgba(255, 255, 255, 0.4);
    color: rgba(255, 255, 255, 0.8);
  }
</style>
