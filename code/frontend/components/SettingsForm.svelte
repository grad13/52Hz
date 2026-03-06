<script lang="ts">
  import type { PresencePosition } from "../lib/settings-store";

  let {
    focusMinutes = $bindable(),
    shortBreakMinutes = $bindable(),
    longBreakMinutes = $bindable(),
    shortBreaksBeforeLong = $bindable(),
    autostartEnabled = false,
    onAutostartChange,
    pauseMediaOnBreak = false,
    onPauseMediaChange,
    hideTrayIcon = false,
    onHideTrayIconChange,
    tickVolume = 0,
    onTickVolumeChange,
    presenceToast = true,
    onPresenceToastChange,
    presencePosition = "top-right" as PresencePosition,
    onPresencePositionChange,
  }: {
    focusMinutes: number;
    shortBreakMinutes: number;
    longBreakMinutes: number;
    shortBreaksBeforeLong: number;
    autostartEnabled: boolean;
    onAutostartChange: (enabled: boolean) => void;
    pauseMediaOnBreak: boolean;
    onPauseMediaChange: (enabled: boolean) => void;
    hideTrayIcon: boolean;
    onHideTrayIconChange: (enabled: boolean) => void;
    tickVolume: number;
    onTickVolumeChange: (volume: number) => void;
    presenceToast: boolean;
    onPresenceToastChange: (enabled: boolean) => void;
    presencePosition: PresencePosition;
    onPresencePositionChange: (pos: PresencePosition) => void;
  } = $props();

  const positions: { value: PresencePosition; label: string }[] = [
    { value: "top-left", label: "↖" },
    { value: "top-right", label: "↗" },
    { value: "bottom-left", label: "↙" },
    { value: "bottom-right", label: "↘" },
  ];
</script>

<section class="form">
  <div class="settings-grid">
    <div class="grid-cell">
      <label for="focus">フォーカス</label>
      <div class="grid-input">
        <input id="focus" type="number" min="1" max="120" bind:value={focusMinutes} />
        <span class="grid-unit">分</span>
      </div>
    </div>
    <div class="grid-cell">
      <label for="short-break">短い休憩</label>
      <div class="grid-input">
        <input id="short-break" type="number" min="1" bind:value={shortBreakMinutes} />
        <span class="grid-unit">分</span>
      </div>
    </div>
    <div class="grid-cell">
      <label for="long-break">長い休憩</label>
      <div class="grid-input">
        <input id="long-break" type="number" min="1" max="30" bind:value={longBreakMinutes} />
        <span class="grid-unit">分</span>
      </div>
    </div>
    <div class="grid-cell">
      <label for="cycles">サイクル</label>
      <div class="grid-input">
        <input id="cycles" type="number" min="1" max="10" bind:value={shortBreaksBeforeLong} />
        <span class="grid-unit">回</span>
      </div>
    </div>
  </div>

  <div class="toggle-grid">
    <div class="toggle-row">
      <span class="toggle-label">メディア自動中断</span>
      <label class="toggle-sm">
        <input
          type="checkbox"
          checked={pauseMediaOnBreak}
          onchange={(e) => onPauseMediaChange(e.currentTarget.checked)}
        />
        <span class="slider"></span>
      </label>
    </div>
    <div class="toggle-row">
      <span class="toggle-label">Tick音</span>
      <input
        class="volume-slider"
        type="range"
        min="0"
        max="1"
        step="0.05"
        value={tickVolume}
        oninput={(e) => onTickVolumeChange(parseFloat(e.currentTarget.value))}
      />
    </div>
    <div class="toggle-row">
      <span class="toggle-label">アイコン非表示</span>
      <label class="toggle-sm">
        <input
          type="checkbox"
          checked={hideTrayIcon}
          onchange={(e) => onHideTrayIconChange(e.currentTarget.checked)}
        />
        <span class="slider"></span>
      </label>
    </div>
    <div class="toggle-row">
      <span class="toggle-label">自動起動</span>
      <label class="toggle-sm">
        <input
          type="checkbox"
          checked={autostartEnabled}
          onchange={(e) => onAutostartChange(e.currentTarget.checked)}
        />
        <span class="slider"></span>
      </label>
    </div>
  </div>

  <div class="presence-section">
    <div class="presence-header">
      <span class="section-label">みんなの存在</span>
      <label class="toggle-sm">
        <input
          type="checkbox"
          checked={presenceToast}
          onchange={(e) => onPresenceToastChange(e.currentTarget.checked)}
        />
        <span class="slider"></span>
      </label>
    </div>
    <div class="presence-options">
      <div class="presence-row">
        <span class="toggle-label">位置</span>
        <div class="pos-buttons">
          {#each positions as pos}
            <button
              class="pos-btn"
              class:active={presencePosition === pos.value}
              onclick={() => onPresencePositionChange(pos.value)}
            >{pos.label}</button>
          {/each}
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  /* 2x2 number input grid */
  .settings-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }

  .grid-cell {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.4rem 0.5rem;
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .grid-cell label {
    font-size: 0.65rem;
    color: var(--text-tertiary);
    letter-spacing: 0.02em;
  }

  .grid-input {
    display: flex;
    align-items: baseline;
    gap: 0.2rem;
  }

  .grid-cell input {
    width: 100%;
    padding: 0.15rem 0;
    font-size: 1rem;
    font-weight: 400;
    text-align: left;
    border: none;
    background: transparent;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .grid-cell input:focus {
    outline: none;
  }

  /* Hide number spinners */
  .grid-cell input::-webkit-inner-spin-button,
  .grid-cell input::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .grid-unit {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  /* 2x2 toggle grid */
  .toggle-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.35rem;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
    padding: 0.3rem 0.5rem;
    border-radius: var(--radius-sm);
    transition: background var(--duration-fast);
  }

  .toggle-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .toggle-label {
    font-size: 0.7rem;
    color: var(--text-secondary);
    flex: 1;
    min-width: 0;
    line-height: 1.2;
  }

  /* Small toggle switch (30x17px) */
  .toggle-sm {
    position: relative;
    display: inline-block;
    width: 30px;
    height: 17px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .toggle-sm input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    inset: 0;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 9px;
    transition: background var(--duration-normal) var(--ease-out);
  }

  .slider::before {
    content: "";
    position: absolute;
    width: 11px;
    height: 11px;
    left: 3px;
    bottom: 3px;
    background: var(--text);
    border-radius: 50%;
    transition: transform var(--duration-normal) var(--ease-out);
  }

  .toggle-sm input:checked + .slider {
    background: var(--success);
  }

  .toggle-sm input:checked + .slider::before {
    transform: translateX(13px);
  }

  .volume-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 90px;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
    flex-shrink: 0;
  }

  .volume-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    background: var(--text);
    border-radius: 50%;
    cursor: pointer;
    transition: background var(--duration-fast);
  }

  .volume-slider::-webkit-slider-thumb:hover {
    background: var(--success);
  }

  /* Presence section */
  .presence-section {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 0.4rem 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .presence-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-label {
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .presence-options {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .presence-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }

  .pos-buttons {
    display: flex;
    gap: 2px;
  }

  .pos-btn {
    padding: 2px 6px;
    font-size: 0.65rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all var(--duration-fast);
  }

  .pos-btn:hover {
    border-color: var(--text-secondary);
    color: var(--text-secondary);
  }

  .pos-btn.active {
    background: var(--success);
    color: #1a1a2e;
    border-color: var(--success);
    font-weight: 600;
  }
</style>
