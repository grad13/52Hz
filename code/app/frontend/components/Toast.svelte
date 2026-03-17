<!-- meta: updated=2026-03-17 06:58 checked=- -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { loadPresenceToast, loadPresencePosition, loadPresenceLevel, loadPresenceMaxToasts, loadPresenceShowIcon, loadPresenceLikeIcon, type PresencePosition, type PresenceLevel, type PresenceLikeIcon } from "../lib/settings-store";
  import { emit } from "@tauri-apps/api/event";
  import { acceptBreak, skipBreakFromFocus } from "../lib/timer";
  import { _ } from "svelte-i18n";

  interface ToastMessage {
    name: string;
    message: string;
  }

  interface ToastItem {
    id: number;
    type: "toast";
    msg: ToastMessage;
    time: string;
    leaving: boolean;
  }

  interface FocusDoneItem {
    id: number;
    type: "focus-done";
    time: string;
    leaving: boolean;
  }

  type StackItem = ToastItem | FocusDoneItem;

  let maxToasts = $state(4);
  const DISPLAY_MS = 180_000; // 3 minutes
  const TOAST_H = 58;
  const FOCUS_DONE_H = 96;
  const GAP = 6;
  const PAD = 8;
  const WIN_W = 276;

  function hashCode(s: string): number {
    let h = 0;
    for (let i = 0; i < s.length; i++) {
      h = (Math.imul(31, h) + s.charCodeAt(i)) | 0;
    }
    return h >>> 0;
  }

  interface PersonaColors {
    body: string;
    iris: string;
  }

  function personaColors(name: string): PersonaColors {
    const h = hashCode(name);
    const bodyHue = h % 360;
    const irisHue = (h >>> 8) % 360;
    return {
      body: `hsl(${bodyHue}, 40%, 25%)`,
      iris: `hsl(${irisHue}, 80%, 55%)`,
    };
  }

  function nowTime(): string {
    const d = new Date();
    return `${d.getHours()}:${String(d.getMinutes()).padStart(2, "0")}`;
  }

  let items: StackItem[] = $state([]);
  let enabled = $state(true);
  let nextId = 0;
  let position: PresencePosition = $state("top-right");
  let level: PresenceLevel = $state("dynamic");
  let hasLikedThisSession: boolean = $state(false);
  let likedId: number | null = $state(null);
  let showIcon = $state(true);
  let likeIcon: PresenceLikeIcon = $state("heart");
  let raised = false; // temporarily raised from back to front
  let shown = false; // track whether window is currently shown (to avoid re-show changing Z-order)
  let needsRaise = true; // first click brings to front, second click dismisses
  let unlistenMsg: (() => void) | null = null;
  let unlistenToggle: (() => void) | null = null;
  let unlistenClick: (() => void) | null = null;
  let unlistenFocusDone: (() => void) | null = null;
  let unlistenPosition: (() => void) | null = null;
  let unlistenLevel: (() => void) | null = null;
  let unlistenMaxToasts: (() => void) | null = null;
  let unlistenShowIcon: (() => void) | null = null;
  let unlistenLikeIcon: (() => void) | null = null;
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
    // Restore level when no focus-done remains (raise was only for focus-done)
    if (!active.some((i) => i.type === "focus-done")) {
      restoreIfNeeded();
    }
    if (active.length === 0) {
      await win.hide();
      shown = false;
      needsRaise = true;
    } else {
      const h = winHeight(active);
      await win.setSize(new LogicalSize(WIN_W, h));
      await emit("presence-reposition", position);
      const hasFocusDone = active.some((i) => i.type === "focus-done");
      if (level === "dynamic" || hasFocusDone) {
        await win.show();
        shown = true;
        await emit("presence-level-change", level);
      } else if (!shown) {
        await win.show();
        shown = true;
        await emit("presence-level-change", level);
      }
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

    // Evict oldest toasts to stay within capacity
    const active = items.filter((i) => !i.leaving);
    const toasts = active.filter((i) => i.type === "toast");
    const excess = toasts.length - (maxToasts - 1); // -1 to make room for new one
    for (let i = 0; i < excess; i++) {
      dismiss(toasts[i].id);
    }

    const id = nextId++;
    const time = nowTime();
    items = [...items, { id, type: "toast", msg, time, leaving: false }];
    syncWindow();

    timers.set(
      id,
      setTimeout(() => dismiss(id), DISPLAY_MS),
    );
  }

  function raise() {
    if (raised) return;
    if (level === "always-back") {
      raised = true;
      emit("presence-level-change", "always-front");
    } else if (level === "dynamic") {
      // dynamic: bring window to front by re-showing (level stays at 0)
      win.show();
    }
    // always-front: do nothing — already in front
  }

  function restoreIfNeeded() {
    if (raised) {
      raised = false;
      emit("presence-level-change", "always-back");
    }
  }

  function handleLike(id: number, e: MouseEvent) {
    e.stopPropagation();
    if (hasLikedThisSession) return;
    hasLikedThisSession = true;
    likedId = id;
  }

  function addFocusDone() {
    hasLikedThisSession = false;
    likedId = null;
    // Remove existing focus-done if any
    const existing = items.find((i) => i.type === "focus-done" && !i.leaving);
    if (existing) dismiss(existing.id);

    const id = nextId++;
    const time = nowTime();
    items = [...items, { id, type: "focus-done", time, leaving: false }];
    syncWindow();
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
    position = await loadPresencePosition();
    level = await loadPresenceLevel();
    maxToasts = await loadPresenceMaxToasts();
    showIcon = await loadPresenceShowIcon();
    likeIcon = await loadPresenceLikeIcon();

    unlistenMsg = (await listen<ToastMessage>("presence-message", (event) => {
      addToast(event.payload);
    })) as unknown as () => void;

    unlistenToggle = (await listen<boolean>("presence-toast-toggle", (event) => {
      enabled = event.payload;
      if (!enabled) dismissAll();
    })) as unknown as () => void;

    // Handle first-click from native global event monitor
    // First click: bring to front. Second click: dismiss oldest toast.
    unlistenClick = (await listen("presence-toast-click", async () => {
      // Two-click interaction only applies in "always-back" mode:
      // first click raises to front, second click dismisses.
      if (level === "always-back" && needsRaise) {
        if (!raised) {
          raise();
        }
        await win.show();
        await emit("presence-level-change", "always-front");
        needsRaise = false;
        return;
      }
      const active = items.filter((i) => !i.leaving);
      // Only dismiss regular toasts on click, not focus-done
      const toasts = active.filter((i) => i.type === "toast");
      if (toasts.length > 0) {
        dismiss(toasts[0].id);
      }
    })) as unknown as () => void;

    unlistenFocusDone = (await listen("focus-done-toast", () => {
      addFocusDone();
    })) as unknown as () => void;

    unlistenPosition = (await listen<string>("presence-position-change", (event) => {
      position = event.payload as PresencePosition;
    })) as unknown as () => void;

    unlistenLevel = (await listen<string>("presence-level-setting", (event) => {
      level = event.payload as PresenceLevel;
      raised = false;
      needsRaise = true;
    })) as unknown as () => void;

    unlistenMaxToasts = (await listen<number>("presence-max-toasts-change", (event) => {
      maxToasts = event.payload;
      // Evict excess toasts immediately
      const active = items.filter((i) => !i.leaving && i.type === "toast");
      const excess = active.length - event.payload;
      for (let i = 0; i < excess; i++) {
        dismiss(active[i].id);
      }
    })) as unknown as () => void;

    unlistenShowIcon = (await listen<boolean>("presence-show-icon-change", (event) => {
      showIcon = event.payload;
    })) as unknown as () => void;

    unlistenLikeIcon = (await listen<string>("presence-like-icon-change", (event) => {
      likeIcon = event.payload as PresenceLikeIcon;
      hasLikedThisSession = false;
      likedId = null;
    })) as unknown as () => void;
  });

  onDestroy(() => {
    unlistenMsg?.();
    unlistenToggle?.();
    unlistenClick?.();
    unlistenFocusDone?.();
    unlistenPosition?.();
    unlistenLevel?.();
    unlistenMaxToasts?.();
    unlistenShowIcon?.();
    unlistenLikeIcon?.();
    for (const t of timers.values()) clearTimeout(t);
  });
</script>

<div
  class="toast-stack"
  class:from-left={position === "top-left" || position === "bottom-left"}
  class:from-bottom={position === "bottom-left" || position === "bottom-right"}
>
  {#each items as item (item.id)}
    {#if item.type === "focus-done"}
      <div class="toast-card focus-done-card" class:leaving={item.leaving}>
        <div class="card-header">
          <span class="label">{$_("focus_done.label")}</span>
          <span class="time">{item.time}</span>
        </div>
        <span class="msg">{$_("focus_done.message")}</span>
        <div class="actions">
          <button class="btn primary" onclick={() => handleAcceptBreak(item.id)}>{$_("focus_done.take_break")}</button>
          <button class="btn" onclick={() => handleSkip(item.id)}>{$_("focus_done.skip")}</button>
        </div>
      </div>
    {:else if item.type === "toast"}
      {@const colors = personaColors(item.msg.name)}
      {@const isLiked = likedId === item.id}
      <div class="toast-card" class:leaving={item.leaving} role="button" tabindex="0" onclick={() => dismiss(item.id)}>
        {#if isLiked && likeIcon !== "none"}
          <span class="bg-art bg-like" class:star={likeIcon === "star"}>{likeIcon === "star" ? "★" : "♥"}</span>
        {:else if showIcon}
          <svg class="bg-art" viewBox="0 0 178 116">
            <g transform="translate(0,116) scale(0.1,-0.1)">
              <g fill={colors.body}><path d="M1324 1074 c-11 -57 2 -102 46 -160 46 -61 50 -95 16 -129 -54 -54 -113 -45 -222 31 -213 149 -370 194 -569 164 -181 -28 -328 -102 -462 -234 -85 -84 -93 -95 -93 -130 0 -33 8 -46 63 -103 34 -36 89 -84 122 -108 66 -47 182 -108 195 -103 14 5 -23 80 -60 121 -48 55 -34 62 46 23 82 -41 151 -99 195 -162 l30 -44 93 0 c197 0 404 87 566 237 64 59 141 166 189 260 35 70 57 88 120 98 57 9 128 73 138 124 5 28 4 32 -8 25 -8 -5 -51 -11 -96 -14 -60 -5 -88 -11 -103 -25 -23 -21 -22 -22 -45 27 -10 23 -35 49 -66 69 -27 17 -54 40 -60 50 -15 29 -27 23 -35 -17z m-495 -191 c55 -20 114 -72 149 -132 24 -41 27 -56 27 -136 0 -73 -4 -98 -22 -130 -75 -140 -248 -199 -386 -132 -129 64 -192 204 -153 340 49 168 215 250 385 190z M390 420 c0 -13 18 -22 24 -11 3 5 -1 11 -9 15 -8 3 -15 1 -15 -4z M450 398 c0 -2 21 -21 48 -43 26 -21 38 -28 27 -16 -19 23 -75 66 -75 59z M470 337 c16 -44 12 -101 -10 -148 -24 -54 -25 -87 -4 -117 23 -33 57 -43 95 -30 85 29 64 156 -45 274 -39 42 -45 45 -36 21z m20 -192 c0 -25 7 -43 25 -59 31 -29 31 -36 1 -36 -47 0 -76 53 -57 103 15 38 31 34 31 -8z"/></g>
              <g fill="#ffffff"><path d="M1314 1105 c-28 -69 -13 -142 46 -213 37 -45 38 -74 4 -101 -49 -38 -96 -25 -221 63 -146 102 -297 156 -436 156 -87 0 -221 -28 -312 -64 -115 -47 -207 -110 -298 -206 -73 -75 -77 -82 -77 -125 0 -41 5 -50 61 -110 34 -36 89 -86 122 -111 71 -53 222 -128 238 -119 10 7 8 24 -7 72 -3 10 1 7 10 -7 21 -34 20 -85 -4 -148 -34 -89 -21 -140 44 -167 40 -17 79 -9 110 21 63 63 25 188 -93 304 -31 30 -42 45 -26 33 37 -26 94 -86 125 -132 l23 -34 126 6 c105 4 140 9 213 34 111 38 221 102 313 179 84 72 132 133 200 257 52 95 68 109 143 127 75 19 142 96 142 166 0 23 -3 25 -22 19 -13 -4 -59 -10 -103 -14 -44 -3 -89 -11 -100 -18 -17 -10 -22 -7 -37 17 -9 16 -41 47 -72 69 -30 22 -57 50 -61 61 -10 31 -34 24 -51 -15z m45 -14 c6 -10 33 -33 60 -50 31 -20 56 -46 66 -69 23 -49 22 -48 45 -27 15 14 43 20 103 25 45 3 88 9 96 14 12 7 13 3 8 -25 -10 -51 -81 -115 -138 -124 -63 -10 -85 -28 -120 -98 -48 -94 -125 -201 -189 -260 -162 -150 -369 -237 -566 -237 l-93 0 -30 44 c-44 63 -113 121 -195 162 -80 39 -94 32 -46 -23 37 -41 74 -116 60 -121 -13 -5 -129 56 -195 103 -33 24 -88 72 -122 108 -55 57 -63 70 -63 103 0 35 8 46 93 130 134 132 281 206 462 234 199 30 356 -15 569 -164 109 -76 168 -85 222 -31 34 34 30 68 -16 129 -44 58 -57 103 -46 160 8 40 20 46 35 17z m-945 -682 c-6 -11 -24 -2 -24 11 0 5 7 7 15 4 8 -4 12 -10 9 -15z m158 -183 c32 -62 39 -105 23 -141 -24 -59 -102 -66 -139 -13 -21 30 -20 63 4 117 22 47 26 104 10 148 -9 24 -3 21 36 -21 25 -27 55 -68 66 -90z M666 899 c-140 -22 -249 -171 -232 -316 28 -240 316 -343 489 -175 66 63 82 104 82 207 0 80 -3 95 -27 136 -67 114 -182 168 -312 148z m169 -33 c58 -27 108 -74 138 -131 17 -31 22 -58 22 -120 0 -70 -4 -87 -30 -134 -31 -56 -88 -106 -148 -128 -18 -6 -61 -12 -97 -12 -262 -2 -375 326 -169 491 77 62 194 76 284 34z M459 153 c-19 -50 10 -103 57 -103 30 0 30 7 -1 36 -18 16 -25 34 -25 59 0 42 -16 46 -31 8z"/></g>
              <g fill="#000000"><path d="M665 881 c-134 -33 -224 -157 -212 -291 6 -73 35 -127 93 -179 144 -126 375 -64 435 116 7 21 9 66 7 109 -4 60 -10 81 -37 122 -38 60 -94 101 -163 119 -57 14 -77 15 -123 4z m163 -117 c43 -35 82 -105 82 -149 0 -46 -44 -127 -85 -154 -80 -54 -192 -34 -252 46 -25 32 -28 45 -28 108 0 63 3 76 29 109 61 81 179 99 254 40z"/></g>
              <g fill={colors.iris}><path d="M668 790 c-46 -14 -85 -48 -107 -93 -87 -180 127 -348 280 -220 81 68 85 198 7 270 -48 45 -118 62 -180 43z M443 615 c0 -22 2 -30 4 -17 2 12 2 30 0 40 -3 9 -5 -1 -4 -23z M914 418 l-19 -23 23 19 c21 18 27 26 19 26 -2 0 -12 -10 -23 -22z"/></g>
            </g>
          </svg>
        {/if}
        <div class="card-header">
          <span class="name">{item.msg.name}</span>
          <span class="header-right">
            {#if likeIcon !== "none" && !hasLikedThisSession}
              <button class="like-btn" onclick={(e) => handleLike(item.id, e)}>{likeIcon === "star" ? "★" : "♥"}</button>
            {/if}
            <span class="time">{item.time}</span>
          </span>
        </div>
        <span class="msg">{item.msg.message}</span>
      </div>
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

  .toast-stack.from-bottom {
    flex-direction: column-reverse;
  }

  .toast-card {
    position: relative;
    overflow: hidden;
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
    animation: slide-in-right 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
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
    animation: slide-out-right 0.3s cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  .from-left .toast-card {
    animation-name: slide-in-left;
  }

  .from-left .toast-card.leaving {
    animation-name: slide-out-left;
  }


  @keyframes slide-in-right {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  @keyframes slide-out-right {
    from { transform: translateX(0); opacity: 1; }
    to { transform: translateX(100%); opacity: 0; }
  }

  @keyframes slide-in-left {
    from { transform: translateX(-100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  @keyframes slide-out-left {
    from { transform: translateX(0); opacity: 1; }
    to { transform: translateX(-100%); opacity: 0; }
  }

  .bg-art {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    height: 44px;
    width: auto;
    opacity: 0.10;
    z-index: 0;
    pointer-events: none;
    user-select: none;
  }

  .bg-like {
    font-size: 40px;
    line-height: 1;
    color: #e8547a;
    opacity: 0.25;
    z-index: 0;
    animation: like-pop 0.3s ease-out;
  }

  .bg-like.star {
    color: #f0c040;
  }

  @keyframes like-pop {
    0% { transform: translate(-50%, -50%) scale(0.5); }
    60% { transform: translate(-50%, -50%) scale(1.2); }
    100% { transform: translate(-50%, -50%) scale(1); }
  }

  .header-right {
    display: flex;
    align-items: baseline;
    gap: 4px;
    flex-shrink: 0;
  }

  .like-btn {
    position: relative;
    z-index: 2;
    background: none;
    border: none;
    padding: 0;
    font-size: 0.6rem;
    color: var(--text-tertiary);
    cursor: pointer;
    opacity: 0.4;
    transition: opacity 0.15s, color 0.15s;
    flex-shrink: 0;
  }

  .like-btn:active {
    opacity: 1;
    color: #e8547a;
  }

  .card-header {
    position: relative;
    z-index: 1;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .time {
    font-size: 0.6rem;
    color: var(--text-tertiary);
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .name, .label {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.02em;
  }

  .msg {
    position: relative;
    z-index: 1;
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
