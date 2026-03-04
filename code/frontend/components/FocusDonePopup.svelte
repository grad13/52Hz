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
    padding: 1.5rem;
    gap: 0.75rem;
  }

  h3 {
    font-size: 1.1rem;
    font-weight: 500;
    color: var(--text);
    margin: 0;
    letter-spacing: 0.03em;
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
    margin-top: 0.25rem;
  }

  .btn {
    flex: 1;
    padding: 0.55rem;
    font-size: 0.85rem;
    font-weight: 500;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-secondary);
    color: var(--text);
    cursor: pointer;
    transition: all var(--duration-normal) var(--ease-out);
  }

  .btn:hover {
    background: var(--bg-elevated);
    border-color: var(--border-hover);
  }

  .btn.primary {
    background: var(--success);
    color: #1a1a2e;
    font-weight: 600;
    border: none;
  }

  .btn.primary:hover {
    background: #5de8b5;
    transform: translateY(-1px);
  }

  .btn.primary:active {
    transform: translateY(0);
  }

  .extend-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-extend {
    padding: 0.3rem 0.8rem;
    font-size: 0.75rem;
    font-weight: 500;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--duration-normal) var(--ease-out);
  }

  .btn-extend:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--border-hover);
    color: var(--text);
  }
</style>
