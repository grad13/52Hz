<script lang="ts">
  let {
    focusMinutes = $bindable(),
    shortBreakSecs = $bindable(),
    longBreakMinutes = $bindable(),
    shortBreaksBeforeLong = $bindable(),
    onSave,
  }: {
    focusMinutes: number;
    shortBreakSecs: number;
    longBreakMinutes: number;
    shortBreaksBeforeLong: number;
    onSave: () => void;
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

  <button class="save-btn" onclick={onSave}>設定を保存</button>
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
