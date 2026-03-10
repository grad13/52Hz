<script lang="ts">
  import type { PresencePosition, PresenceLevel, PresenceLikeIcon } from "../lib/settings-store";
  import type { CassetteInfo } from "../lib/timer";
  import { _ } from "svelte-i18n";

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
    presenceLevel = "dynamic" as PresenceLevel,
    onPresenceLevelChange,
    presenceMaxToasts = 4,
    onPresenceMaxToastsChange,
    presenceShowIcon = true,
    onPresenceShowIconChange,
    presenceLikeIcon = "heart" as PresenceLikeIcon,
    onPresenceLikeIconChange,
    cassettes = [] as CassetteInfo[],
    currentCassette = "",
    onCassetteChange,
    onOpenCassetteFolder,
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
    presenceLevel: PresenceLevel;
    onPresenceLevelChange: (level: PresenceLevel) => void;
    presenceMaxToasts: number;
    onPresenceMaxToastsChange: (n: number) => void;
    presenceShowIcon: boolean;
    onPresenceShowIconChange: (v: boolean) => void;
    presenceLikeIcon: PresenceLikeIcon;
    onPresenceLikeIconChange: (v: PresenceLikeIcon) => void;
    cassettes: CassetteInfo[];
    currentCassette: string;
    onCassetteChange: (path: string) => void;
    onOpenCassetteFolder: () => void;
  } = $props();

  let cassetteListOpen = $state(false);

  const currentCassetteTitle = $derived(
    cassettes.find(c => c.path === currentCassette)?.title ?? "default"
  );

  const positions: { value: PresencePosition; label: string }[] = [
    { value: "top-left", label: "↖" },
    { value: "top-right", label: "↗" },
    { value: "bottom-left", label: "↙" },
    { value: "bottom-right", label: "↘" },
  ];

  const likeOptions: { value: PresenceLikeIcon; label: string }[] = $derived([
    { value: "heart" as PresenceLikeIcon, label: "♥" },
    { value: "star" as PresenceLikeIcon, label: "★" },
    { value: "none" as PresenceLikeIcon, label: $_("settings.presence_like_none") },
  ]);
</script>

<section class="form">
  <div class="settings-grid">
    <div class="grid-cell">
      <label for="focus">{$_("settings.focus")}</label>
      <div class="grid-input">
        <input id="focus" type="number" min="1" max="120" bind:value={focusMinutes} />
        <span class="grid-unit">{$_("settings.unit_min")}</span>
      </div>
    </div>
    <div class="grid-cell">
      <label for="short-break">{$_("settings.short_break")}</label>
      <div class="grid-input">
        <input id="short-break" type="number" min="1" bind:value={shortBreakMinutes} />
        <span class="grid-unit">{$_("settings.unit_min")}</span>
      </div>
    </div>
    <div class="grid-cell">
      <label for="long-break">{$_("settings.long_break")}</label>
      <div class="grid-input">
        <input id="long-break" type="number" min="1" max="30" bind:value={longBreakMinutes} />
        <span class="grid-unit">{$_("settings.unit_min")}</span>
      </div>
    </div>
    <div class="grid-cell">
      <label for="cycles">{$_("settings.cycles")}</label>
      <div class="grid-input">
        <input id="cycles" type="number" min="1" max="10" bind:value={shortBreaksBeforeLong} />
        <span class="grid-unit">{$_("settings.unit_times")}</span>
      </div>
    </div>
  </div>

  <div class="toggle-grid">
    <div class="toggle-row">
      <span class="toggle-label">{$_("settings.auto_pause_media")}</span>
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
      <span class="toggle-label">{$_("settings.tick_sound")}</span>
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
      <span class="toggle-label">{$_("settings.hide_icon")}</span>
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
      <span class="toggle-label">{$_("settings.autostart")}</span>
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
      <span class="section-label">{$_("settings.presence")}</span>
      <label class="toggle-sm">
        <input
          type="checkbox"
          checked={presenceToast}
          onchange={(e) => onPresenceToastChange(e.currentTarget.checked)}
        />
        <span class="slider"></span>
      </label>
    </div>
    {#if presenceToast}
      <div class="presence-options">
        <div class="sub-group">
          <span class="sub-group-label">{$_("settings.presence_general")}</span>
          <div class="presence-row">
            <span class="toggle-label">{$_("settings.presence_show_icon")}</span>
            <label class="toggle-sm">
              <input
                type="checkbox"
                checked={presenceShowIcon}
                onchange={(e) => onPresenceShowIconChange(e.currentTarget.checked)}
              />
              <span class="slider"></span>
            </label>
          </div>
          <div class="presence-row">
            <span class="toggle-label">{$_("settings.presence_like")}</span>
            <div class="like-buttons">
              {#each likeOptions as opt}
                <button
                  class="like-btn"
                  class:active={presenceLikeIcon === opt.value}
                  onclick={() => onPresenceLikeIconChange(opt.value)}
                >{opt.label}</button>
              {/each}
            </div>
          </div>
        </div>
        <div class="sub-group">
          <span class="sub-group-label">{$_("settings.presence_message")}</span>
          <div class="presence-row">
            <span class="toggle-label">{$_("settings.presence_position")}</span>
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
          <div class="presence-row">
            <span class="toggle-label">{$_("settings.presence_order")}</span>
            <div class="level-buttons">
              <button
                class="level-btn"
                class:active={presenceLevel === "always-front"}
                onclick={() => onPresenceLevelChange("always-front")}
              >{$_("settings.presence_order_front")}</button>
              <button
                class="level-btn"
                class:active={presenceLevel === "dynamic"}
                onclick={() => onPresenceLevelChange("dynamic")}
              >{$_("settings.presence_order_dynamic")}</button>
              <button
                class="level-btn"
                class:active={presenceLevel === "always-back"}
                onclick={() => onPresenceLevelChange("always-back")}
              >{$_("settings.presence_order_back")}</button>
            </div>
          </div>
          <div class="presence-row">
            <span class="toggle-label">{$_("settings.presence_max")}</span>
            <div class="limit-buttons">
              {#each [2, 3, 4, 5] as n}
                <button
                  class="limit-btn"
                  class:active={presenceMaxToasts === n}
                  onclick={() => onPresenceMaxToastsChange(n)}
                >{n}</button>
              {/each}
            </div>
          </div>
        </div>
      </div>
      <div class="sub-group">
        <span class="sub-group-label">{$_("settings.presence_cassette")}</span>
        <div class="cassette-card" onclick={() => cassetteListOpen = !cassetteListOpen} onkeydown={(e) => { if (e.key === 'Enter') cassetteListOpen = !cassetteListOpen; }} role="button" tabindex="0">
          <div class="cassette-tape">
            <div class="cassette-reel"></div>
            <div class="cassette-reel"></div>
          </div>
          <div class="cassette-info">
            <div class="cassette-title">{currentCassetteTitle}</div>
          </div>
          <span class="cassette-chevron" class:open={cassetteListOpen}>›</span>
        </div>
        {#if cassetteListOpen}
          <div class="cassette-list">
            {#each cassettes as c}
              <button
                class="cassette-item"
                class:active={c.path === currentCassette}
                onclick={() => { onCassetteChange(c.path); cassetteListOpen = false; }}
              >
                <span class="cassette-item-radio"></span>
                <span class="cassette-item-name">{c.title}</span>
              </button>
            {/each}
            <button class="cassette-folder-btn" onclick={(e) => { e.stopPropagation(); onOpenCassetteFolder(); }}>
              📂 {$_("settings.presence_cassette_folder")}
            </button>
          </div>
        {/if}
      </div>
    {/if}
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

  .sub-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-top: 0.15rem;
    border-top: 1px solid var(--border);
    padding-top: 0.35rem;
  }

  .sub-group:first-child {
    border-top: none;
    margin-top: 0;
    padding-top: 0;
  }

  .sub-group-label {
    font-size: 0.6rem;
    font-weight: 600;
    color: var(--text-tertiary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .sub-group .presence-row {
    padding-left: 0.5rem;
  }

  .presence-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }

  .pos-buttons, .level-buttons, .limit-buttons, .like-buttons {
    display: flex;
    gap: 2px;
  }

  .pos-btn, .level-btn, .limit-btn, .like-btn {
    padding: 2px 6px;
    font-size: 0.65rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all var(--duration-fast);
  }

  .pos-btn:hover, .level-btn:hover, .limit-btn:hover, .like-btn:hover {
    border-color: var(--text-secondary);
    color: var(--text-secondary);
  }

  .pos-btn.active, .level-btn.active, .limit-btn.active, .like-btn.active {
    background: var(--success);
    color: #1a1a2e;
    border-color: var(--success);
    font-weight: 600;
  }

  /* Cassette card */
  .cassette-card {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.5rem;
    margin-left: 0.5rem;
    background: rgba(108, 92, 231, 0.08);
    border: 1px solid rgba(108, 92, 231, 0.15);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: border-color 150ms;
  }

  .cassette-card:hover {
    border-color: rgba(108, 92, 231, 0.3);
  }

  .cassette-tape {
    width: 28px;
    height: 18px;
    background: var(--bg-elevated);
    border-radius: 3px;
    border: 1px solid rgba(108, 92, 231, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
    flex-shrink: 0;
  }

  .cassette-reel {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    border: 1px solid var(--accent-light);
  }

  .cassette-info {
    flex: 1;
    min-width: 0;
  }

  .cassette-title {
    font-size: 0.7rem;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cassette-chevron {
    font-size: 0.7rem;
    color: var(--text-tertiary);
    transition: transform 150ms;
    flex-shrink: 0;
  }

  .cassette-chevron.open {
    transform: rotate(90deg);
  }

  .cassette-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin-left: 0.5rem;
    margin-top: 0.15rem;
  }

  .cassette-item {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 3px 6px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 150ms;
    font-size: 0.65rem;
    color: var(--text-tertiary);
    border: none;
    background: transparent;
    text-align: left;
  }

  .cassette-item:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .cassette-item.active {
    color: var(--accent-light);
  }

  .cassette-item-radio {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 1px solid var(--text-tertiary);
    flex-shrink: 0;
  }

  .cassette-item.active .cassette-item-radio {
    border-color: var(--accent-light);
    background: var(--accent-light);
    box-shadow: inset 0 0 0 2px var(--bg);
  }

  .cassette-item-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cassette-folder-btn {
    padding: 3px 6px;
    font-size: 0.6rem;
    color: var(--text-tertiary);
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    border-top: 1px solid var(--border);
    margin-top: 2px;
    padding-top: 5px;
    transition: color 150ms;
  }

  .cassette-folder-btn:hover {
    color: var(--text-secondary);
  }
</style>
