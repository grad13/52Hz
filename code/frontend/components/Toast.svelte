<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { loadPresenceToast } from "../lib/settings-store";

  interface ToastMessage {
    name: string;
    message: string;
  }

  interface ToastItem {
    id: number;
    msg: ToastMessage;
    leaving: boolean;
  }

  const MAX_TOASTS = 10;
  const DISPLAY_MS = 180_000; // 3 minutes
  const TOAST_H = 58;
  const GAP = 6;
  const PAD = 8;
  const WIN_W = 276;

  let items: ToastItem[] = $state([]);
  let enabled = $state(true);
  let nextId = 0;
  let unlistenMsg: (() => void) | null = null;
  let unlistenToggle: (() => void) | null = null;
  let unlistenClick: (() => void) | null = null;
  const win = getCurrentWindow();
  const timers = new Map<number, ReturnType<typeof setTimeout>>();

  function winHeight(n: number): number {
    if (n <= 0) return 0;
    return n * TOAST_H + (n - 1) * GAP + PAD * 2;
  }

  async function syncWindow() {
    const active = items.filter((i) => !i.leaving).length;
    if (active === 0) {
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
      const timer = timers.get(item.id);
      if (timer) clearTimeout(timer);
    }
    timers.clear();
    items = [];
    syncWindow();
  }

  function addToast(msg: ToastMessage) {
    if (!enabled) return;

    // Evict oldest if at capacity
    const active = items.filter((i) => !i.leaving);
    if (active.length >= MAX_TOASTS) {
      dismiss(active[0].id);
    }

    const id = nextId++;
    items = [...items, { id, msg, leaving: false }];
    syncWindow();

    timers.set(
      id,
      setTimeout(() => dismiss(id), DISPLAY_MS),
    );
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
    // (JS onclick doesn't fire on the activation click for Accessory apps)
    unlistenClick = (await listen("presence-toast-click", () => {
      const active = items.filter((i) => !i.leaving);
      if (active.length > 0) {
        dismiss(active[0].id);
      }
    })) as unknown as () => void;
  });

  onDestroy(() => {
    unlistenMsg?.();
    unlistenToggle?.();
    unlistenClick?.();
    for (const t of timers.values()) clearTimeout(t);
  });
</script>

<div class="toast-stack">
  {#each items as item (item.id)}
    <button class="toast-card" class:leaving={item.leaving} onclick={() => dismiss(item.id)}>
      <span class="name">{item.msg.name}</span>
      <span class="msg">{item.msg.message}</span>
    </button>
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

  .name {
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
</style>
