<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { loadPresenceToast } from "../lib/settings-store";
  import { acceptBreak, skipBreakFromFocus } from "../lib/timer";

  interface ToastMessage {
    name: string;
    message: string;
  }

  interface ToastItem {
    id: number;
    type: "toast";
    msg: ToastMessage;
    leaving: boolean;
  }

  interface FocusDoneItem {
    id: number;
    type: "focus-done";
    leaving: boolean;
  }

  type StackItem = ToastItem | FocusDoneItem;

  const MAX_TOASTS = 10;
  const DISPLAY_MS = 180_000; // 3 minutes
  const TOAST_H = 58;
  const FOCUS_DONE_H = 100;
  const GAP = 6;
  const PAD = 8;
  const WIN_W = 276;

  let items: StackItem[] = $state([]);
  let enabled = $state(true);
  let nextId = 0;
  let unlistenMsg: (() => void) | null = null;
  let unlistenToggle: (() => void) | null = null;
  let unlistenClick: (() => void) | null = null;
  let unlistenFocusDone: (() => void) | null = null;
  const win = getCurrentWindow();
  const timers = new Map<number, ReturnType<typeof setTimeout>>();

  function itemHeight(item: StackItem): number {
    return item.type === "focus-done" ? FOCUS_DONE_H : TOAST_H;
  }

  function winHeight(active: StackItem[]): number {
    if (active.length === 0) return 0;
    const total = active.reduce((sum, i) => sum + itemHeight(i), 0);
    return total + (active.length - 1) * GAP + PAD * 2;
  }

  async function syncWindow() {
    const active = items.filter((i) => !i.leaving);
    if (active.length === 0) {
      await win.hide();
    } else {
      await win.setSize(new LogicalSize(WIN_W, winHeight(active)));
      await win.show();
    }
  }

  function dismiss(id: number) {
    const timer = timers.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.delete(id);
    }
    items = items.map((i) => (i.id === id ? { ...i, leaving: true } : i));
    setTimeout(() => {
      items = items.filter((i) => i.id !== id);
      syncWindow();
    }, 350);
  }

  function dismissAll() {
    for (const item of items) {
      if (item.type === "toast") {
        const timer = timers.get(item.id);
        if (timer) clearTimeout(timer);
        timers.delete(item.id);
      }
    }
    items = items.filter((i) => i.type !== "toast");
    syncWindow();
  }

  function addToast(msg: ToastMessage) {
    if (!enabled) return;

    // Evict oldest if at capacity
    const active = items.filter((i) => !i.leaving);
    if (active.length >= MAX_TOASTS) {
      const oldest = active.find((i) => i.type === "toast");
      if (oldest) dismiss(oldest.id);
    }

    const id = nextId++;
    items = [...items, { id, type: "toast", msg, leaving: false }];
    syncWindow();

    timers.set(
      id,
      setTimeout(() => dismiss(id), DISPLAY_MS),
    );
  }

  function addFocusDone() {
    // Remove existing focus-done if any
    const existing = items.find((i) => i.type === "focus-done" && !i.leaving);
    if (existing) dismiss(existing.id);

    const id = nextId++;
    items = [...items, { id, type: "focus-done", leaving: false }];
    syncWindow();
    // No auto-dismiss — user must choose an action
  }

  async function handleAcceptBreak(id: number) {
    await acceptBreak();
    dismiss(id);
  }

  async function handleSkip(id: number) {
    await skipBreakFromFocus();
    dismiss(id);
  }

  onMount(async () => {
    enabled = await loadPresenceToast();

    unlistenMsg = (await listen<ToastMessage>("presence-message", (event) => {
      addToast(event.payload);
    })) as unknown as () => void;

    unlistenToggle = (await listen<boolean>("presence-toast-toggle", (event) => {
      enabled = event.payload;
      if (!enabled) dismissAll();
    })) as unknown as () => void;

    // Handle first-click from native global event monitor
    unlistenClick = (await listen("presence-toast-click", () => {
      const active = items.filter((i) => !i.leaving);
      // Only dismiss regular toasts on first-click, not focus-done
      const toasts = active.filter((i) => i.type === "toast");
      if (toasts.length > 0) {
        dismiss(toasts[0].id);
      }
    })) as unknown as () => void;

    unlistenFocusDone = (await listen("focus-done-toast", () => {
      addFocusDone();
    })) as unknown as () => void;
  });

  onDestroy(() => {
    unlistenMsg?.();
    unlistenToggle?.();
    unlistenClick?.();
    unlistenFocusDone?.();
    for (const t of timers.values()) clearTimeout(t);
  });
</script>

<div class="toast-stack">
  {#each items as item (item.id)}
    {#if item.type === "focus-done"}
      <div class="toast-card focus-done-card" class:leaving={item.leaving}>
        <span class="label">セッション完了</span>
        <span class="msg">お疲れ様です！次はどうしますか？</span>
        <div class="actions">
          <button class="btn primary" onclick={() => handleAcceptBreak(item.id)}>休憩する</button>
          <button class="btn" onclick={() => handleSkip(item.id)}>スキップ</button>
        </div>
      </div>
    {:else}
      <button class="toast-card" class:leaving={item.leaving} onclick={() => dismiss(item.id)}>
        <span class="name">{item.msg.name}</span>
        <span class="msg">{item.msg.message}</span>
      </button>
    {/if}
  {/each}
</div>

<style>
  .toast-stack {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
  }

  .toast-card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    outline: none;
    animation: slide-in 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    transition: opacity 0.15s;
  }

  .toast-card:hover {
    opacity: 0.7;
  }

  .focus-done-card {
    gap: 6px;
    padding: 10px 12px;
    cursor: default;
  }

  .focus-done-card:hover {
    opacity: 1;
  }

  .toast-card.leaving {
    animation: slide-out 0.3s cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  @keyframes slide-in {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  @keyframes slide-out {
    from {
      transform: translateX(0);
      opacity: 1;
    }
    to {
      transform: translateX(100%);
      opacity: 0;
    }
  }

  .name, .label {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.02em;
  }

  .msg {
    font-size: 0.78rem;
    color: var(--text);
    line-height: 1.35;
  }

  .actions {
    display: flex;
    gap: 6px;
    margin-top: 2px;
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

</style>
