---
updated: 2026-03-16 07:20
checked: -
Deprecated: -
Format: spec-v2.1
Source: frontend/App.svelte
---

# App.svelte spec

## 0. Meta

| Field | Value |
|-------|-------|
| Runtime | JS-ESM (Svelte 5 component) |
| File | `code/app/frontend/App.svelte` |
| Role | Application root / view router |
| Lines | 17 |

### Overview

Root component of the 52Hz app. Reads the URL query parameter `view` and
exclusively mounts either the break overlay (`BreakOverlay`), focus-done popup
(`FocusDonePopup`), or tray panel (`TrayPanel`).
Routing is a static branch determined by the URL that the Tauri backend sets
when creating the window — no runtime view transitions occur.

### Dependencies

| Import | Purpose |
|--------|---------|
| `./app.css` | Global CSS (custom properties, reset) |
| `./components/TrayPanel.svelte` | Tray panel view component |
| `./components/BreakOverlay.svelte` | Break overlay view component |
| `./components/FocusDonePopup.svelte` | Focus-done popup view component |

---

## 1. Contract

### Props

None. As the root component, it receives no external props.

### Exports

None. No functions or variables are exposed externally.

### Events (dispatch)

None.

### Slots / Snippets

None.

### Implicit Inputs

| Input | Type | Description |
|-------|------|-------------|
| `window.location.search` | `string` | URL query parameters. Set by Tauri when creating the window |

---

## 2. State

### Local Variables (non-reactive)

| Name | Type | Initial Value | Description |
|------|------|---------------|-------------|
| `params` | `URLSearchParams` | `new URLSearchParams(window.location.search)` | URL query parameter parser |
| `view` | `string \| null` | `params.get("view")` | View identifier. `"break"`, `"focus-done"`, or `null` |

### Reactive State

None. `view` is `const` and does not participate in Svelte's reactivity system.

### View State Machine

```
[Init]
  │
  ├─ view === "break"      ──→ BreakOverlay is mounted
  │
  ├─ view === "focus-done" ──→ FocusDonePopup is mounted
  │
  └─ otherwise (incl. null) ──→ TrayPanel is mounted
```

- Transition occurs only once at component initialization
- No runtime state changes

---

## 3. Side Effects

### Tauri Events

App.svelte does not handle Tauri events directly.
Event subscriptions are the responsibility of child components (TrayPanel, BreakOverlay).

### Global CSS

`import "./app.css"` applies the following global styles:

| Effect | Details |
|--------|---------|
| CSS custom properties | Theme variables (`--bg`, `--text`, `--accent`, etc.) defined on `:root` |
| Box model reset | `* { margin: 0; padding: 0; box-sizing: border-box }` |
| Full-height layout | `html, body, #app { height: 100%; width: 100% }` |
| Text selection disabled | `user-select: none` |
| System font | `-apple-system, BlinkMacSystemFont, ...` |

---

## 4. View Branching

### View Branching Logic

```
Input: window.location.search
Process:
  1. Parse query string with URLSearchParams
  2. Get value of "view" key
  3. Value is "break"      → Render BreakOverlay
     Value is "focus-done" → Render FocusDonePopup
     Otherwise (incl. null) → Render TrayPanel
```

### Decision Criteria

<!-- Tests: 4-1, 4-2 -->

| ID | Query Example | `view` Value | Displayed View | Test ID |
|----|--------------|-------------|----------------|---------|
| 4-1 | `?view=break` | `"break"` | BreakOverlay | 4-1 |
| 4-2 | (no parameter) | `null` | TrayPanel | 4-2 |
| 4-3 | `?view=focus-done` | `"focus-done"` | FocusDonePopup | 4-3 |
| - | `?view=other` | `"other"` | TrayPanel | - |
| - | `?view=` | `""` | TrayPanel | - |

- `4-1`: When `?view=break`, BreakOverlay component is shown and TrayPanel is not displayed
- `4-2`: When no parameter, TrayPanel component is shown and BreakOverlay is not displayed
- `4-3`: When `?view=focus-done`, FocusDonePopup component is shown and neither TrayPanel nor BreakOverlay is displayed
- `?view=other` and `?view=` cases are not covered by tests but logically fall back to TrayPanel

---

## 5. Notes

### Design Intent

- App.svelte is intentionally kept thin. It handles only routing, delegating business logic to child components
- Tauri v2 manages multiple windows in a single process, loading the same frontend bundle with different URL parameters per window
- Tray panel window: no parameter → TrayPanel
- Break overlay window: `?view=break` → BreakOverlay
- Focus-done popup window: `?view=focus-done` → FocusDonePopup

### Constraints & Assumptions

- `view` is `const`, so switching views after mount is impossible (intentional)
- Adding a new view requires extending the `{#if}` chain and setting the URL parameter on the Tauri backend side
- Dependency on `window.location.search` assumes a browser/WebView environment (no SSR)

### Future Extension Points

- To add a new view, add `{:else if view === "xxx"}`
- If the number of views grows, consider switching to dynamic imports (`{#await import(...)}`) or map-based routing
