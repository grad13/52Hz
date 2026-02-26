<script lang="ts">
  let { paused, onTogglePause, onQuit }: {
    paused: boolean;
    onTogglePause: () => void;
    onQuit: () => void;
  } = $props();
</script>

<section class="controls">
  <button class="control-btn" onclick={onTogglePause}>
    {paused ? "▶ 再開" : "⏸ 一時停止"}
  </button>
  <button class="control-btn quit-btn" onclick={onQuit}>
    ✕ 終了
  </button>
</section>

<style>
  .controls {
    display: flex;
    justify-content: center;
  }

  .control-btn {
    padding: 0.5rem 1.5rem;
    font-size: 0.9rem;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    background: var(--accent);
    color: var(--text);
    cursor: pointer;
    transition: background 0.2s;
  }

  .control-btn:hover {
    background: var(--accent-light);
  }

  .quit-btn {
    background: transparent;
    border-color: var(--danger);
    color: var(--danger);
  }

  .quit-btn:hover {
    background: var(--danger);
    color: #fff;
  }
</style>
