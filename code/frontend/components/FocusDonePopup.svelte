<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { acceptBreak, extendFocus, skipBreakFromFocus } from "../lib/timer";
  import { _ } from "svelte-i18n";

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
  <div class="card">
    <span class="label">{$_("focus_done.label")}</span>
    <span class="message">{$_("focus_done.message")}</span>

    <div class="actions">
      <button class="btn primary" onclick={handleAcceptBreak}>{$_("focus_done.take_break")}</button>
      <button class="btn" onclick={handleSkip}>{$_("focus_done.skip")}</button>
    </div>

    <div class="extend-actions">
      <button class="btn-extend" onclick={() => handleExtend(60)}>{$_("focus_done.extend_1")}</button>
      <button class="btn-extend" onclick={() => handleExtend(180)}>{$_("focus_done.extend_3")}</button>
      <button class="btn-extend" onclick={() => handleExtend(300)}>{$_("focus_done.extend_5")}</button>
    </div>
  </div>
</div>

<style>
  .popup {
    padding: 8px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .label {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.02em;
  }

  .message {
    font-size: 0.78rem;
    color: var(--text);
    line-height: 1.35;
  }

  .actions {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }

  .btn {
    flex: 1;
    padding: 5px 0;
    font-size: 0.75rem;
    font-weight: 500;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.06);
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
  }

  .extend-actions {
    display: flex;
    gap: 4px;
    justify-content: center;
  }

  .btn-extend {
    padding: 3px 8px;
    font-size: 0.68rem;
    font-weight: 500;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-extend:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--border-hover);
    color: var(--text);
  }
</style>
