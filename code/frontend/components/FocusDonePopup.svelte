<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { acceptBreak, extendFocus, skipBreakFromFocus } from "../lib/timer";

  async function handleAcceptBreak() {
    await acceptBreak();
    getCurrentWindow().close();
  }

  async function handleExtend(secs: number) {
    await extendFocus(secs);
    getCurrentWindow().close();
  }

  async function handleSkip() {
    await skipBreakFromFocus();
    getCurrentWindow().close();
  }
</script>

<div class="popup">
  <h3>Focus Complete</h3>
  <p class="message">お疲れ様です！次はどうしますか？</p>

  <div class="actions">
    <button class="btn primary" onclick={handleAcceptBreak}>休憩する</button>
    <button class="btn" onclick={handleSkip}>スキップ</button>
  </div>

  <div class="extend-actions">
    <button class="btn-extend" onclick={() => handleExtend(60)}>+1分</button>
    <button class="btn-extend" onclick={() => handleExtend(180)}>+3分</button>
    <button class="btn-extend" onclick={() => handleExtend(300)}>+5分</button>
  </div>
</div>

<style>
  .popup {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 1rem;
    gap: 0.6rem;
  }

  h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
    margin: 0;
  }

  .message {
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    width: 100%;
  }

  .btn {
    flex: 1;
    padding: 0.5rem;
    font-size: 0.85rem;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--text);
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .btn:hover {
    opacity: 0.8;
  }

  .btn.primary {
    background: var(--success);
    color: #1a1a2e;
    font-weight: 600;
    border: none;
  }

  .extend-actions {
    display: flex;
    gap: 0.4rem;
  }

  .btn-extend {
    padding: 0.3rem 0.8rem;
    font-size: 0.8rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-extend:hover {
    background: rgba(255, 255, 255, 0.08);
  }
</style>
