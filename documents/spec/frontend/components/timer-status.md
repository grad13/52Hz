---
Created: 2026-02-26
Updated: 2026-03-03
Verified: -
Deprecated: -
Format: spec-v2.1
Source: frontend/components/TimerStatus.svelte
---

# TimerStatus Component Spec

## 1. Overview

A Svelte 5 component that visually displays the current timer state.
A pure display component consisting of three elements: phase name, remaining time, and a paused badge.

- Runtime: JS-ESM (Svelte 5 component)
- File: `code/app/frontend/components/TimerStatus.svelte`
- Lines: 72

## 2. Responsibility

- Display the timer's phase name
- Display the remaining time in countdown format
- Display a visual badge when paused
- Display cycle progress as dots

### Out of Scope

- Timer logic management (responsibility of the parent component)
- Formatting the remaining time (receives a pre-formatted string from the caller)
- Handling user interactions (responsibility of TimerControls)

## 3. Public Interface

### 3.1 Props

| Prop name | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `phaseLabel` | `string` | Yes | - | Display name of the current phase |
| `remaining` | `string` | Yes | - | Pre-formatted remaining time string |
| `paused` | `boolean` | Yes | - | Paused state flag |
| `cycleCompleted` | `number` | Yes | - | Number of short breaks completed in the current cycle. Determines how many dots to fill |
| `cycleTotal` | `number` | Yes | - | Total number of dots in the cycle (short break count + 1 long break). Used for dot generation count |
| `isLongBreak` | `boolean` | Yes | - | Whether the current phase is a long break. When true, all dots are filled |

All props are declared with `$props()`. No default values (all required).

### 3.2 Events

None.

### 3.3 Slots

None.

## 4. Internal State

None. A stateless component with no local state.

## 5. Rendering Spec

### 5.1 DOM Structure

```html
<section class="status">
  <div class="phase">{phaseLabel}</div>
  <div class="time">{remaining}</div>
  <!-- Shown only when paused === true -->
  <div class="paused-badge">Paused</div>
  <!-- Cycle dots (always displays cycleTotal dots) -->
  <div class="cycle-dots">
    {#each Array(cycleTotal) as _, i}
      <span class="dot" class:filled={isLongBreak || i < cycleCompleted}></span>
    {/each}
  </div>
</section>
```

### 5.2 Conditional Rendering

| Condition | Element | Behavior |
|-----------|---------|----------|
| `paused === true` | `.paused-badge` | Inserted into DOM, displays `"Paused"` (`status.paused_badge`) |
| `paused === false` | `.paused-badge` | Removed from DOM |
| `isLongBreak === true` or `i < cycleCompleted` | `.filled` class on `.dot` | Dot displayed in filled color |

### 5.3 Text Content

| Element | Content | Source |
|---------|---------|--------|
| `.phase` | Phase name | `phaseLabel` prop |
| `.time` | Remaining time | `remaining` prop |
| `.paused-badge` | `"Paused"` (`status.paused_badge`) | Hardcoded (Japanese literal in source) |

## 6. Style Spec

Scoped CSS (automatically scoped by the Svelte compiler).

### 6.1 CSS Class Definitions

| Class name | Purpose | Key properties |
|------------|---------|----------------|
| `.status` | Root container | `text-align: center`, `padding: 0.8rem 0` |
| `.phase` | Phase name | `font-size: 0.85rem`, `color: var(--text-secondary)`, `margin-bottom: 0.3rem` |
| `.time` | Remaining time | `font-size: 2.8rem`, `font-weight: 300`, `font-variant-numeric: tabular-nums`, `letter-spacing: 0.05em` |
| `.paused-badge` | Paused badge | `display: inline-block`, `background: var(--danger)`, `color: #fff`, `border-radius: 4px`, `font-size: 0.75rem` |
| `.cycle-dots` | Dot container | `display: flex`, `justify-content: center`, `gap: 0.4rem`, `margin-top: 0.5rem` |
| `.dot` | Individual dot (unfilled) | `width: 8px`, `height: 8px`, `border-radius: 50%`, `background: rgba(255,255,255,0.15)`, `transition: background 0.3s` |
| `.dot.filled` | Individual dot (filled) | `background: var(--accent-light)` |

### 6.2 CSS Custom Property Dependencies

| Property name | Used by class | Purpose |
|---------------|--------------|---------|
| `--text-secondary` | `.phase` | Phase name text color |
| `--danger` | `.paused-badge` | Paused badge background color |
| `--accent-light` | `.dot.filled` | Filled dot background color |

These custom properties are expected to be supplied externally (from global CSS or theme definitions).

### 6.3 Design Intent

- `font-variant-numeric: tabular-nums` is specified on `.time` to prevent layout jitter caused by varying digit widths during countdown
- `font-weight: 300` on `.time` maintains a light visual impression even at large font sizes
- `.paused-badge` uses `inline-block` for natural placement within center-aligned text

## 7. Lifecycle

None. No lifecycle processing such as `onMount`, `onDestroy`, `$effect` exists.

## 8. Side Effects

None.

## 9. Usage Pattern

### 9.1 Invocation from Parent Component

```svelte
<!-- TrayPanel.svelte (line 86) -->
<TimerStatus {phaseLabel} {remaining} {paused} {cycleCompleted} {cycleTotal} {isLongBreak} />
```

Shorthand prop binding passes same-named variables from the parent scope directly.

## 10. External Dependencies

None. No module imports exist.

## 11. Constraints & Assumptions

- `remaining` must be a pre-formatted string from the caller (this component does not perform formatting)
- CSS custom properties `--text-secondary`, `--danger`, `--accent-light` must be defined globally
- The `"Paused"` (`status.paused_badge`) text is hardcoded in Japanese (no i18n support)
