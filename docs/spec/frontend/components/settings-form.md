---
Created: 2026-02-26
Updated: 2026-03-08
Verified: -
Deprecated: -
Format: spec-v2.1
Source: frontend/components/SettingsForm.svelte
Origin: _documents/spec/frontend/components/settings-form.md
Test: tests/unit/frontend/components/settings-form.test.ts
---

# SettingsForm Component Spec

## 1. Overview

A timer settings input form UI component.
A logic-free, purely Presentational component that arranges 4 numeric input fields and 4 toggles in a 2x2 grid, with a volume slider and a dedicated presence section.

- Runtime: JS-ESM (Svelte 5 component)
- File: `code/frontend/components/SettingsForm.svelte`

## 2. Responsibility

- **Display** timer settings values and **accept user input**
- Sync values with the parent component via **two-way binding**

### Out of Scope

- Value validation (except for HTML native min/max)
- Settings persistence
- Conversion between display units and internal units

## 3. UI Structure & Interaction <!-- Tests 3-1 ~ 3-11 -->

Tests use a flat 3-x numbering scheme (3-1 ~ 3-11), verifying input field display, field constraints, toggles, sliders, and the presence section.

### 3-1: Input Field Display

4 input fields are displayed with the following labels.

| Label | HTML id | Bound prop |
|-------|---------|------------|
| Focus | `focus` | `focusMinutes` |
| Short break | `short-break` | `shortBreakMinutes` |
| Long break | `long-break` | `longBreakMinutes` |
| Cycles | `cycles` | `shortBreaksBeforeLong` |

Each field contains a label (`<label>`), a numeric input (`<input type="number">`), and a unit display (`<span>`).

### 3-2: Focus Time Field Constraints

| Attribute | Value |
|-----------|-------|
| id | `focus` |
| type | `number` |
| min | `1` |
| max | `120` |
| unit | min |

### 3-3: Short Break Field Constraints

| Attribute | Value |
|-----------|-------|
| id | `short-break` |
| type | `number` |
| min | `1` |
| unit | min |

### 3-4: Long Break Field Constraints

| Attribute | Value |
|-----------|-------|
| id | `long-break` |
| type | `number` |
| min | `1` |
| max | `30` |
| unit | min |

### 3-5: Cycles Before Long Break Field Constraints

| Attribute | Value |
|-----------|-------|
| id | `cycles` |
| type | `number` |
| min | `1` |
| max | `10` |
| unit | times |

### 3-6: Toggle Grid

4 toggle items are arranged in a 2x2 grid (`toggle-grid`).

| Label | Type | Bound prop | Callback |
|-------|------|------------|----------|
| Auto-pause media | `checkbox` (toggle switch) | `pauseMediaOnBreak` | `onPauseMediaChange` |
| Tick sound | `range` (volume slider) | `tickVolume` | `onTickVolumeChange` |
| Hide icon | `checkbox` (toggle switch) | `hideTrayIcon` | `onHideTrayIconChange` |
| Auto-start | `checkbox` (toggle switch) | `autostartEnabled` | `onAutostartChange` |

- Each toggle uses a toggle switch UI (CSS slider display, 30x17px)
- Invokes the corresponding callback on `onchange`

### 3-7: Auto-Pause Media Toggle

- Label: "Auto-pause media" (`settings.pause_media`)
- `checked` is controlled by the `pauseMediaOnBreak` prop
- Invokes `onPauseMediaChange(checked)` callback on `onchange`

### 3-8: Hide Icon Toggle

- Label: "Hide icon" (`settings.hide_icon`)
- `checked` is controlled by the `hideTrayIcon` prop
- Invokes `onHideTrayIconChange(checked)` callback on `onchange`

### 3-9: Tick Sound Volume Slider

- Label: "Tick sound" (`settings.tick_sound`)
- `<input type="range">` (class: `volume-slider`)
- `min="0"`, `max="1"`, `step="0.05"`
- Width: 90px fixed
- `value` is controlled by the `tickVolume` prop
- Invokes `onTickVolumeChange(parseFloat(value))` callback on `oninput`

### 3-10: Auto-Start Toggle

- Label: "Auto-start" (`settings.autostart`)
- `checked` is controlled by the `autostartEnabled` prop
- Invokes `onAutostartChange(checked)` callback on `onchange`

### 3-11: Presence Section

Enclosed in a dedicated `.presence-section` container.

#### Header

- Label: "Everyone's presence (On52Hz)" (`settings.presence_header`) (class: `section-label`)
- Toggle switch: controlled by `presenceToast` prop
- Invokes `onPresenceToastChange(checked)` callback on `onchange`

#### Presence Options (Conditionally Displayed)

When `presenceToast` is `false`, the following option group (`.presence-options`) is hidden (`{#if presenceToast}`).

#### Subgroup "General"

##### User Icon Display Toggle

- Label: "Show user icon" (`settings.show_user_icon`)
- `checked` is controlled by the `presenceShowIcon` prop
- Invokes `onPresenceShowIconChange(checked)` callback on `onchange`

##### Like Feature Buttons (`.like-buttons`)

- Label: "Like feature" (`settings.like_feature`)
- 3 buttons: heart (`heart`), star (`star`), none (`none`)
- The active button receives the `.active` class (matches `presenceLikeIcon`)
- Invokes `onPresenceLikeIconChange(value)` callback on `onclick`

#### Subgroup "Messages"

##### Position Buttons (`.pos-buttons`)

- Label: "Position" (`settings.position`)
- 4 buttons: top-left (`top-left`), top-right (`top-right`), bottom-left (`bottom-left`), bottom-right (`bottom-right`)
- The active button receives the `.active` class (matches `presencePosition`)
- Invokes `onPresencePositionChange(value)` callback on `onclick`

##### Level Buttons (`.level-buttons`)

- Label: "Order" (`settings.level`)
- 3 buttons: Always front (`always-front`), Dynamic (`dynamic`), Always back (`always-back`)
- The active button receives the `.active` class (matches `presenceLevel`)
- Invokes `onPresenceLevelChange(value)` callback on `onclick`

##### Max Count Buttons (`.limit-buttons`)

- Label: "Max count" (`settings.max_count`)
- 4 buttons: [2] [3] [4] [5]
- The active button receives the `.active` class (matches `presenceMaxToasts`)
- Invokes `onPresenceMaxToastsChange(n)` callback on `onclick`

### 3.sup: Validation (Not Subject to Testing)

- Only browser-native constraints via HTML `<input type="number">` `min`/`max` attributes
- No JavaScript validation logic exists within the component
- When out-of-range values are directly entered, behavior depends on the browser

## 4. Interface (Not Subject to Testing)

### 4.1 Props

| Prop name | Type | Required | Bindable | Default | Description |
|-----------|------|----------|----------|---------|-------------|
| `focusMinutes` | `number` | Yes | Yes | - | Focus time (minutes) |
| `shortBreakMinutes` | `number` | Yes | Yes | - | Short break time (minutes) |
| `longBreakMinutes` | `number` | Yes | Yes | - | Long break time (minutes) |
| `shortBreaksBeforeLong` | `number` | Yes | Yes | - | Number of short breaks before a long break |
| `autostartEnabled` | `boolean` | No | No | `false` | Current auto-start state |
| `onAutostartChange` | `(enabled: boolean) => void` | Yes | No | - | Callback on auto-start toggle change |
| `pauseMediaOnBreak` | `boolean` | No | No | `false` | Current media pause on break state |
| `onPauseMediaChange` | `(enabled: boolean) => void` | Yes | No | - | Callback on media pause toggle change |
| `hideTrayIcon` | `boolean` | No | No | `false` | Current hide tray icon state |
| `onHideTrayIconChange` | `(enabled: boolean) => void` | Yes | No | - | Callback on hide tray icon toggle change |
| `tickVolume` | `number` | No | No | `0` | Tick sound volume (0-1) |
| `onTickVolumeChange` | `(volume: number) => void` | Yes | No | - | Callback on volume slider change |
| `presenceToast` | `boolean` | No | No | `true` | Presence toast display enabled/disabled |
| `onPresenceToastChange` | `(enabled: boolean) => void` | Yes | No | - | Callback on presence toast toggle change |
| `presencePosition` | `PresencePosition` | No | No | `"top-right"` | Presence toast display position |
| `onPresencePositionChange` | `(pos: PresencePosition) => void` | Yes | No | - | Callback on presence position change |
| `presenceLevel` | `PresenceLevel` | No | No | `"dynamic"` | Presence toast display level |
| `onPresenceLevelChange` | `(level: PresenceLevel) => void` | Yes | No | - | Callback on presence level change |
| `presenceMaxToasts` | `number` | No | No | `4` | Maximum number of presence toasts displayed |
| `onPresenceMaxToastsChange` | `(n: number) => void` | Yes | No | - | Callback on max toast count change |
| `presenceShowIcon` | `boolean` | No | No | `true` | User icon display enabled/disabled |
| `onPresenceShowIconChange` | `(v: boolean) => void` | Yes | No | - | Callback on user icon display toggle change |
| `presenceLikeIcon` | `PresenceLikeIcon` | No | No | `"heart"` | Like feature icon type |
| `onPresenceLikeIconChange` | `(v: PresenceLikeIcon) => void` | Yes | No | - | Callback on like icon change |

### 4.2 Bindings

The 4 numeric props are declared with `$bindable()`, enabling two-way binding from the parent component via `bind:propName`. Input field value changes are immediately reflected in the parent's state.

## 5. Style Spec (Not Subject to Testing)

### 5.1 CSS Variable Dependencies

| Variable name | Purpose |
|---------------|---------|
| `--text-secondary` | Label / section label color |
| `--text-tertiary` | Grid cell label / unit text / inactive button color |
| `--bg-secondary` | Grid cell background color |
| `--text` | Input field / slider thumb text color |
| `--border` | Grid cell / presence section / button border color |
| `--success` | Toggle switch ON state / active button background color |
| `--radius-md`, `--radius-sm` | Border radius for various elements |
| `--duration-fast`, `--duration-normal` | Transition durations |
| `--ease-out` | Transition easing |

### 5.2 Layout

- Root `.form`: flexbox column, gap 0.5rem
- `.settings-grid`: 2x2 CSS grid (`grid-template-columns: 1fr 1fr`), gap 0.5rem
- `.grid-cell`: flexbox column, background `--bg-secondary`, with border
- `.toggle-grid`: 2x2 CSS grid, gap 0.35rem
- `.toggle-row`: flexbox row, `justify-content: space-between`
- `.presence-section`: flexbox column, bordered container

### 5.3 Interactions

- Toggle row hover: background `rgba(255, 255, 255, 0.03)`
- Volume slider thumb hover: color changes to `--success`
- Position/level/limit/like button hover: border and text change to `--text-secondary`
- Active button: background `--success`, text color `#1a1a2e`, font-weight 600

## 6. Usage Example

```svelte
<SettingsForm
  bind:focusMinutes
  bind:shortBreakMinutes
  bind:longBreakMinutes
  bind:shortBreaksBeforeLong
  {autostartEnabled}
  onAutostartChange={handleAutostartChange}
  {pauseMediaOnBreak}
  onPauseMediaChange={handlePauseMediaChange}
  {hideTrayIcon}
  onHideTrayIconChange={handleHideTrayIconChange}
  {tickVolume}
  onTickVolumeChange={handleTickVolumeChange}
  {presenceToast}
  onPresenceToastChange={handlePresenceToastChange}
  {presencePosition}
  onPresencePositionChange={handlePresencePositionChange}
  {presenceLevel}
  onPresenceLevelChange={handlePresenceLevelChange}
  {presenceMaxToasts}
  onPresenceMaxToastsChange={handlePresenceMaxToastsChange}
  {presenceShowIcon}
  onPresenceShowIconChange={handlePresenceShowIconChange}
  {presenceLikeIcon}
  onPresenceLikeIconChange={handlePresenceLikeIconChange}
/>
```

Used in the above format within the parent component (`TrayPanel.svelte`).

## 7. Dependencies

### 7.1 External Dependencies

- `PresencePosition`, `PresenceLevel`, `PresenceLikeIcon` types imported from `../lib/settings-store`

### 7.2 Internal Dependencies

- Parent: `TrayPanel.svelte` (sole consumer)
- CSS variables depend on the app-wide theme definition

## 8. Constraints & Assumptions

- Uses Svelte 5 runes API (`$props()`, `$bindable()`)
- Scoped CSS (Svelte `<style>` tag) keeps styles contained within the component
- Assumes operation within a Tauri WebView (browser compatibility is not considered)
