<script lang="ts">
  let {
    focusMinutes = $bindable(),
    shortBreakMinutes = $bindable(),
    longBreakMinutes = $bindable(),
    shortBreaksBeforeLong = $bindable(),
    autostartEnabled = false,
    onAutostartChange,
    pauseMediaOnBreak = false,
    onPauseMediaChange,
  }: {
    focusMinutes: number;
    shortBreakMinutes: number;
    longBreakMinutes: number;
    shortBreaksBeforeLong: number;
    autostartEnabled: boolean;
    onAutostartChange: (enabled: boolean) => void;
    pauseMediaOnBreak: boolean;
    onPauseMediaChange: (enabled: boolean) => void;
  } = $props();
</script>

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
        min="1"
        bind:value={shortBreakMinutes}
      />
      <span class="unit">分</span>
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

  <div class="field">
    <label for="pause-media">休憩時にメディアを一時停止</label>
    <label class="toggle">
      <input
        id="pause-media"
        type="checkbox"
        checked={pauseMediaOnBreak}
        onchange={(e) => onPauseMediaChange(e.currentTarget.checked)}
      />
      <span class="slider"></span>
    </label>
  </div>

  <div class="field">
    <label for="autostart">ログイン時に自動起動</label>
    <label class="toggle">
      <input
        id="autostart"
        type="checkbox"
        checked={autostartEnabled}
        onchange={(e) => onAutostartChange(e.currentTarget.checked)}
      />
      <span class="slider"></span>
    </label>
  </div>

</section>

<style>
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

  .toggle {
    position: relative;
    display: inline-block;
    width: 36px;
    height: 20px;
    cursor: pointer;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    inset: 0;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    transition: background 0.2s;
  }

  .slider::before {
    content: "";
    position: absolute;
    width: 14px;
    height: 14px;
    left: 3px;
    bottom: 3px;
    background: var(--text);
    border-radius: 50%;
    transition: transform 0.2s;
  }

  .toggle input:checked + .slider {
    background: var(--success);
  }

  .toggle input:checked + .slider::before {
    transform: translateX(16px);
  }

</style>
