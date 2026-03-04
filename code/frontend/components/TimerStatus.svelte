<script lang="ts">
  let { remaining, paused, cycleCompleted, cycleTotal, isLongBreak, todaySessions, onTogglePause }: {
    remaining: string;
    paused: boolean;
    cycleCompleted: number;
    cycleTotal: number;
    isLongBreak: boolean;
    todaySessions: number;
    onTogglePause: () => void;
  } = $props();

  let hovering = $state(false);

  const pulsePointsRunning =
    "0,40 20,40 30,40 35,38 40,36 44,38 48,40 " +
    "52,40 54,42 56,40 58,6 61,62 64,40 " +
    "68,40 74,34 80,32 86,34 92,40 100,40 " +
    "100,40 120,40 130,40 135,38 140,36 144,38 148,40 " +
    "152,40 154,42 156,40 158,6 161,62 164,40 " +
    "168,40 174,34 180,32 186,34 192,40 200,40 " +
    "200,40 220,40 230,40 235,38 240,36 244,38 248,40 " +
    "252,40 254,42 256,40 258,6 261,62 264,40 " +
    "268,40 274,34 280,32 286,34 292,40 300,40 " +
    "300,40 320,40 330,40 335,38 340,36 344,38 348,40 " +
    "352,40 354,42 356,40 358,6 361,62 364,40 " +
    "368,40 374,34 380,32 386,34 392,40 400,40 " +
    "400,40 420,40 430,40 435,38 440,36 444,38 448,40 " +
    "452,40 454,42 456,40 458,6 461,62 464,40 " +
    "468,40 474,34 480,32 486,34 492,40 500,40 " +
    "500,40 520,40 530,40 535,38 540,36 544,38 548,40 " +
    "552,40 554,42 556,40 558,6 561,62 564,40 " +
    "568,40 574,34 580,32 586,34 592,40 600,40";

  const pulsePointsPaused =
    "0,40 20,40 30,40 35,39.6 40,39.2 44,39.6 48,40 " +
    "52,40 54,40.4 56,40 58,33.2 61,44.4 64,40 " +
    "68,40 74,38.8 80,38.4 86,38.8 92,40 100,40 " +
    "100,40 120,40 130,40 135,39.6 140,39.2 144,39.6 148,40 " +
    "152,40 154,40.4 156,40 158,33.2 161,44.4 164,40 " +
    "168,40 174,38.8 180,38.4 186,38.8 192,40 200,40 " +
    "200,40 220,40 230,40 235,39.6 240,39.2 244,39.6 248,40 " +
    "252,40 254,40.4 256,40 258,33.2 261,44.4 264,40 " +
    "268,40 274,38.8 280,38.4 286,38.8 292,40 300,40 " +
    "300,40 320,40 330,40 335,39.6 340,39.2 344,39.6 348,40 " +
    "352,40 354,40.4 356,40 358,33.2 361,44.4 364,40 " +
    "368,40 374,38.8 380,38.4 386,38.8 392,40 400,40 " +
    "400,40 420,40 430,40 435,39.6 440,39.2 444,39.6 448,40 " +
    "452,40 454,40.4 456,40 458,33.2 461,44.4 464,40 " +
    "468,40 474,38.8 480,38.4 486,38.8 492,40 500,40 " +
    "500,40 520,40 530,40 535,39.6 540,39.2 544,39.6 548,40 " +
    "552,40 554,40.4 556,40 558,33.2 561,44.4 564,40 " +
    "568,40 574,38.8 580,38.4 586,38.8 592,40 600,40";
</script>

<section class="status" class:paused>
  <div class="timer-hover" class:hovering role="button" tabindex="0" onclick={onTogglePause} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onTogglePause(); }} onmouseenter={() => hovering = true} onmouseleave={() => hovering = false}>
    <div class="time">{remaining}</div>
    <div class="pulse-container">
      <svg class="pulse-line" width="600" height="80" viewBox="0 0 600 80">
        <polyline
          fill="none"
          stroke="var(--pulse)"
          stroke-width={paused ? "1.5" : "2"}
          stroke-linejoin="round"
          stroke-linecap="round"
          points={paused ? pulsePointsPaused : pulsePointsRunning}
        />
      </svg>
    </div>
    <div class="hover-btn">
      <button tabindex="-1">{paused ? "▶ 再開" : "⏸ 一時停止"}</button>
    </div>
  </div>
  <div class="cycle-dots">
    {#each Array(cycleTotal) as _, i}
      <span class="dot" class:filled={isLongBreak || i < cycleCompleted}></span>
    {/each}
    {#if todaySessions > 0}
      <span class="session-badge">{todaySessions}回完了</span>
    {/if}
  </div>
</section>

<style>
  .status {
    text-align: center;
    padding: 0.25rem 0;
  }

  .timer-hover {
    position: relative;
    cursor: pointer;
    overflow: hidden;
    outline: none;
    border: none;
  }

  .time {
    position: relative;
    z-index: 1;
    font-size: 6rem;
    font-weight: 200;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.08em;
    line-height: 1;
    padding: 0.5rem 0;
    transition: opacity 200ms ease-in-out 201ms;
  }

  .status.paused .time {
    opacity: 0.5;
  }

  /* Pulse ECG line */
  .pulse-container {
    width: 100%;
    height: 80px;
    overflow: hidden;
    position: absolute;
    top: 50%;
    left: 0;
    transform: translateY(-50%);
    opacity: 0.35;
    pointer-events: none;
  }

  .pulse-container svg {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
  }

  .pulse-line {
    animation: pulse-scroll 7s linear infinite;
  }

  @keyframes pulse-scroll {
    from { transform: translateX(0); }
    to { transform: translateX(-200px); }
  }

  /* Hover: show pause/resume button */
  .hover-btn {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 200ms ease-in-out;
    pointer-events: none;
  }

  .hover-btn button {
    padding: 0.6rem 2rem;
    font-size: 2.25rem;
    font-weight: 500;
    border: none;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    transition: color var(--duration-normal) var(--ease-out);
  }

  .hover-btn button:hover {
    color: var(--accent-light);
  }

  .timer-hover.hovering .time {
    opacity: 0;
    transition: opacity 200ms ease-in-out;
  }

  .timer-hover.hovering .hover-btn {
    opacity: 1;
    pointer-events: auto;
    transition: opacity 200ms ease-in-out 201ms;
  }

  /* Cycle dots + session badge */
  .cycle-dots {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    transition: all var(--duration-normal) var(--ease-out);
  }

  .dot.filled {
    background: var(--accent-light);
    box-shadow: 0 0 6px var(--accent-glow);
  }

  .session-badge {
    font-size: 0.65rem;
    color: var(--text-tertiary);
    margin-left: 0.5rem;
    letter-spacing: 0.02em;
  }
</style>
