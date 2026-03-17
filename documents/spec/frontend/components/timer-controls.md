---
updated: 2026-03-16 07:20
checked: -
Deprecated: -
Format: spec-v2.1
Source: frontend/components/TimerControls.svelte
---

# TimerControls.svelte spec

## 1. Overview

A presentational component providing UI button groups for timer pause/resume and stop operations.

- Runtime: JS-ESM (Svelte 5 component)
- Path: `code/app/frontend/components/TimerControls.svelte`
- Lines: 49

## 2. Responsibility

- Render the timer pause/resume toggle button
- Render the timer stop button
- Switch the toggle button label based on the `paused` state
- Propagate button clicks to the parent component via callbacks

## 3. Out of Scope

- Timer state management (responsibility of parent `TrayPanel`)
- Tauri command invocation (executed via parent)
- Holding or changing settings values

## 4. Interface

### 4.1 Props (Input)

| Prop name | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `paused` | `boolean` | Yes | - | Whether the timer is paused. Shows `"Resume"` (`controls.resume`) when `true`, `"Pause"` (`controls.pause`) when `false` |
| `onTogglePause` | `() => void` | Yes | - | Callback invoked when the pause/resume button is pressed |
| `onStop` | `() => void` | Yes | - | Callback invoked when the stop button is pressed |

### 4.2 Output (Events / Slots)

None. Communicates using the callback props pattern.

## 5. Internal State

None. A completely stateless component.

## 6. DOM Structure

```
<section class="controls">
  <button class="control-btn" onclick={onTogglePause}>
    {paused ? "▶ Resume" : "⏸ Pause"}
  </button>
  <button class="control-btn stop-btn" onclick={onStop}>
    ■ Stop
  </button>
</section>
```

### 6.1 Element List

| Element | Class | Role |
|---------|-------|------|
| `<section>` | `controls` | Root container. Centered with Flexbox |
| `<button>` (1) | `control-btn` | Pause/resume toggle |
| `<button>` (2) | `control-btn stop-btn` | Timer stop |

## 7. Behavior

### 7.1 Label Switching

| `paused` value | Toggle button display |
|----------------|----------------------|
| `true` | `"▶ Resume"` (`controls.resume`) |
| `false` | `"⏸ Pause"` (`controls.pause`) |

The stop button always displays `"■ Stop"` (`controls.stop`).

### 7.2 Event Flow

```
User click (toggle button)  ->  onTogglePause() callback invoked
User click (stop button)    ->  onStop() callback invoked
```

## 8. Styles

### 8.1 Scope

Svelte component-scoped CSS. No external stylesheet dependencies.

### 8.2 CSS Custom Property Dependencies

| Property name | Usage | Purpose |
|---------------|-------|---------|
| `--accent` | `.control-btn` background | Toggle button background color |
| `--accent-light` | `.control-btn:hover` background | Toggle button hover background color |
| `--text` | `.control-btn` color | Toggle button text color |
| `--danger` | `.stop-btn` border-color, color | Stop button accent color |

### 8.3 Layout

- `.controls`: `display: flex; justify-content: center`
- `.control-btn`: `padding: 0.5rem 1.5rem`, `border-radius: 6px`, `transition: background 0.2s`
- `.stop-btn`: `background: transparent` (normal), `background: var(--danger); color: #fff` (hover)

## 9. Dependencies

### 9.1 External Dependencies

None. No imports other than the Svelte runtime.

### 9.2 Consumers

| Component | File | Invocation |
|-----------|------|------------|
| `TrayPanel` | `frontend/components/TrayPanel.svelte` (L87) | `<TimerControls {paused} onTogglePause={handleTogglePause} onStop={resetTimer} />` |

## 10. Testing Guidelines

| # | Test content | Category |
|---|-------------|----------|
| 1 | When `paused=false`, `"⏸ Pause"` (`controls.pause`) is displayed | Rendering |
| 2 | When `paused=true`, `"▶ Resume"` (`controls.resume`) is displayed | Rendering |
| 3 | Stop button always displays `"■ Stop"` (`controls.stop`) | Rendering |
| 4 | Clicking the toggle button calls `onTogglePause` | Event |
| 5 | Clicking the stop button calls `onStop` | Event |
| 6 | Two buttons are rendered | Structure |

## 11. Design Notes

- Uses Svelte 5's `$props()` rune with the callback props pattern. Does not use `createEventDispatcher` from Svelte 4 and earlier.
- Completely stateless, making it highly testable.
- CSS custom properties enable theme swapping.
