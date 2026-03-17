---
updated: 2026-03-16 07:20
checked: -
Deprecated: -
Format: spec-v2.1
Source: frontend/components/TrayPanel.svelte
---

# TrayPanel Component Spec

## 1. Overview

Root component of the tray popup window. Integrates timer state display, pause/resume/stop operations, and settings editing/saving into a single panel.

- **Runtime**: JS-ESM (Svelte 5 component)
- **File**: `code/app/frontend/components/TrayPanel.svelte`

## 2. Responsibility

| # | Responsibility | Description |
|---|---------------|-------------|
| R1 | Real-time timer state display | Receives `timer-tick` events from the backend and propagates remaining time, phase, and paused state to child components |
| R2 | Timer control | Executes pause/resume toggle and app quit via Tauri commands |
| R3 | Settings management | Loads settings from the persistent store, and after user editing, reflects them to the backend + saves to store |
| R4 | Child component composition | Arranges TimerStatus / TimerControls / SettingsForm and manages the data flow |
| R5 | Auto-start management | Uses the `@tauri-apps/plugin-autostart` JS API to toggle login auto-start ON/OFF |
| R6 | Daily session count display | Fetches today's completed session count from the backend and displays it. Re-fetches on each phase change to keep the value current |
| R7 | Media pause setting management | Loads/saves the media pause on break setting from/to the persistent store |
| R8 | Tray icon visibility setting management | Loads/saves the tray icon show/hide setting from/to the persistent store and reflects it to the backend via Tauri command |
| R9 | Tick volume management | Loads/saves the tick sound volume setting from/to the persistent store |
| R10 | Presence display position management | Loads/saves the presence toast display position setting from/to the persistent store and reflects it to the window and backend via `emitTo` + `emit` |
| R11 | Presence level management | Loads/saves the presence toast display level setting from/to the persistent store and reflects it to the window and backend via `emitTo` + `emit` |

## 3. Public Interface

### 3.1 Props

None. This component is used as the root of the tray window and does not receive props from the outside.

### 3.2 Events (emit)

None.

### 3.3 Slots

None.

## 4. Internal State

| Variable name | Type | Initial value | Update trigger | Purpose |
|--------------|------|---------------|----------------|---------|
| `timerState` | `TimerState \| null` | `null` | On `timer-tick` event reception | Holds the backend timer state |
| `remaining` | `string` | `"--:--"` | On `timer-tick` event reception | Formatted remaining time (MM:SS) |
| `phaseLabel` | `string` | `"Focus"` | On `timer-tick` event reception | Japanese phase label |
| `paused` | `boolean` | `false` | On `timer-tick` reception, `togglePause` execution | Paused state |
| `focusMinutes` | `number` | `20` | On settings load, user input | Focus time (minutes) |
| `shortBreakMinutes` | `number` | `1` | On settings load, user input | Short break (minutes) |
| `settingsLoaded` | `boolean` | `false` | Set to `true` on `loadSavedSettings()` completion | Flag to suppress saving on initial mount |
| `saveTimeout` | `ReturnType<typeof setTimeout> \| null` | `null` | Debounce timer management in `$effect` | Auto-save debounce timer |
| `longBreakMinutes` | `number` | `3` | On settings load, user input | Long break (minutes) |
| `shortBreaksBeforeLong` | `number` | `3` | On settings load, user input | Number of short breaks before long break |
| `autostartEnabled` | `boolean` | `false` | Fetched with `isEnabled()` on mount, on toggle change | Current auto-start state |
| `pauseMediaOnBreak` | `boolean` | `false` | Fetched with `loadPauseMediaOnBreak()` on mount, on toggle change | Media pause on break setting |
| `hideTrayIcon` | `boolean` | `false` | Fetched with `loadHideTrayIcon()` on mount, on toggle change | Hide tray icon setting |
| `tickVolume` | `number` | `0` | Fetched with `loadTickVolume()` on mount, on slider change | Tick sound volume (0-1) |
| `presencePosition` | `PresencePosition` | `"top-right"` | Fetched with `loadPresencePosition()` on mount, on selection change | Presence toast display position |
| `presenceLevel` | `PresenceLevel` | `"dynamic"` | Fetched with `loadPresenceLevel()` on mount, on selection change | Presence toast display level |
| `todaySessions` | `number` | `0` | Fetched with `getTodaySessions()` on mount, on `phase-changed` event reception | Today's completed session count |
| `cycleCompleted` | `number` | `0` | On `timer-tick` reception (`$derived`) | Number of short breaks completed in the current cycle (`timerState?.short_break_count ?? 0`) |
| `cycleTotal` | `number` | `1` | On `timer-tick` reception (`$derived`) | Total slots in the cycle (`(timerState?.settings.short_breaks_before_long ?? 0) + 1`) |
| `isLongBreak` | `boolean` | `false` | On `timer-tick` reception (`$derived`) | Whether the current phase is a long break (`timerState?.phase === "LongBreak" ?? false`) |

## 5. Phase Label Mapping

| Backend value | Display label |
|--------------|---------------|
| `"Focus"` | `"Focusing"` (`phase.focus`) |
| `"ShortBreak"` | `"Short break"` (`phase.short_break`) |
| `"LongBreak"` | `"Long break"` (`phase.long_break`) |

When an unknown value is received, the backend value is displayed as-is (fallback).

## 6. Lifecycle

### 6.1 On Mount (onMount)

```
1. loadSavedSettings()
   a. Load saved settings from settings-store
   b. If settings exist:
      - Update internal state setting values
      - Reflect to backend with updateSettings()
   c. If settings are null, do nothing (keep default values)
2. Fetch auto-start state with isAutostartEnabled() (false on failure)
3. Load media pause setting with loadPauseMediaOnBreak()
3b. Load hide tray icon setting with loadHideTrayIcon()
3c. Load presence display position setting with loadPresencePosition()
3d. Load presence level setting with loadPresenceLevel()
4. Fetch today's session count with getTodaySessions() and initialize todaySessions
5. Fetch current timer state with getTimerState()
6. Initialize display with handleTick()
7. Start event subscription with onTimerTick(handleTick) and retain the unlisten function
8. Subscribe to phase change events with onPhaseChanged(), calling getTodaySessions() each time to update todaySessions. Retain the unlisten function
```

### 6.2 During Operation

- `handleTick()` is called on each `timer-tick` event reception, updating all display state
- `getTodaySessions()` is called on each `phase-changed` event reception, updating `todaySessions`

### 6.3 On Unmount (onDestroy)

- Call the retained unlisten function to unsubscribe from the `timer-tick` event
- Call the retained unlisten function to unsubscribe from the `phase-changed` event

## 7. User Actions

### 7.1 Pause/Resume Toggle

```
handleTogglePause():
  1. Call the togglePause() Tauri command
  2. Immediately update paused with the return value (new paused state)
```

### 7.2 Quit App

```
Call quitApp() Tauri command directly (imported from timer.ts)
```

### 7.3 Auto-Start Toggle

```
handleAutostartChange(enabled):
  1. Call enableAutostart() if enabled is true, disableAutostart() if false
  2. Re-fetch actual state with isAutostartEnabled() and update autostartEnabled
  3. On error: re-fetch state with isAutostartEnabled() (fallback: false)
```

### 7.4 Media Pause Toggle

```
handlePauseMediaChange(enabled):
  1. Immediately update pauseMediaOnBreak to enabled
  2. Save to persistent store with savePauseMediaOnBreak(enabled)
```

### 7.5 Hide Tray Icon Toggle

```
handleHideTrayIconChange(enabled):
  1. Immediately update hideTrayIcon to enabled
  2. Save to persistent store with saveHideTrayIcon(enabled)
  3. Reflect to backend with setTrayIconVisible(!enabled) (hide=true -> visible=false)
```

### 7.6 Timer Reset (Stop)

```
Call resetTimer() Tauri command directly (imported from timer.ts).
Resets the timer to its initial state and restarts from the beginning of the Focus phase.
```

### 7.7 Settings Save

```
handleSaveSettings():
  1. Build a DisplaySettings object from current display values (all in minutes)
  2. Convert to backend format (all in seconds) with toTimerSettings()
  3. Reflect to backend with updateSettings()
  4. Save to persistent store with saveSettings()

$effect (auto-save):
  - Subscribes to focusMinutes, shortBreakMinutes, longBreakMinutes, shortBreaksBeforeLong
  - Does nothing while settingsLoaded is false (ignores initial value changes on mount)
  - On change detection, clears existing saveTimeout and invokes handleSaveSettings() with 500ms debounce
```

### 7.8 Tick Volume Change

```
handleTickVolumeChange(volume):
  1. Immediately update tickVolume to volume
  2. Save to persistent store with saveTickVolume(volume)
```

### 7.9 Presence Display Position Change

```
handlePresencePositionChange(pos):
  1. Immediately update presencePosition to pos
  2. Save to persistent store with savePresencePosition(pos)
  3. Notify presence window with emitTo("presence-toast", "presence-position-change", pos)
  4. Notify backend with emit("presence-position-change", pos)
```

### 7.10 Presence Level Change

```
handlePresenceLevelChange(level):
  1. Immediately update presenceLevel to level
  2. Save to persistent store with savePresenceLevel(level)
  3. Notify presence window with emitTo("presence-toast", "presence-level-setting", level)
  4. Notify backend with emit("presence-level-change", level)
```

## 8. Child Component Composition

### 8.1 TimerStatus

| Prop | Type | Binding | Source |
|------|------|---------|--------|
| `phaseLabel` | `string` | One-way | Internal state `phaseLabel` |
| `remaining` | `string` | One-way | Internal state `remaining` |
| `paused` | `boolean` | One-way | Internal state `paused` |
| `cycleCompleted` | `number` | One-way | `$derived` variable `cycleCompleted` |
| `cycleTotal` | `number` | One-way | `$derived` variable `cycleTotal` |
| `isLongBreak` | `boolean` | One-way | `$derived` variable `isLongBreak` |

### 8.2 TimerControls

| Prop | Type | Binding | Source |
|------|------|---------|--------|
| `paused` | `boolean` | One-way | Internal state `paused` |
| `onTogglePause` | `() => void` | Callback | `handleTogglePause` |
| `onStop` | `() => void` | Callback | `resetTimer` |

### 8.3 SettingsForm

| Prop | Type | Binding | Source |
|------|------|---------|--------|
| `focusMinutes` | `number` | `bind:` two-way | Internal state `focusMinutes` |
| `shortBreakMinutes` | `number` | `bind:` two-way | Internal state `shortBreakMinutes` |
| `longBreakMinutes` | `number` | `bind:` two-way | Internal state `longBreakMinutes` |
| `shortBreaksBeforeLong` | `number` | `bind:` two-way | Internal state `shortBreaksBeforeLong` |
| `autostartEnabled` | `boolean` | One-way | Internal state `autostartEnabled` |
| `onAutostartChange` | `(enabled: boolean) => void` | Callback | `handleAutostartChange` |
| `pauseMediaOnBreak` | `boolean` | One-way | Internal state `pauseMediaOnBreak` |
| `onPauseMediaChange` | `(enabled: boolean) => void` | Callback | `handlePauseMediaChange` |
| `hideTrayIcon` | `boolean` | One-way | Internal state `hideTrayIcon` |
| `onHideTrayIconChange` | `(enabled: boolean) => void` | Callback | `handleHideTrayIconChange` |
| `tickVolume` | `number` | One-way | Internal state `tickVolume` |
| `onTickVolumeChange` | `(volume: number) => void` | Callback | `handleTickVolumeChange` |
| `presencePosition` | `PresencePosition` | One-way | Internal state `presencePosition` |
| `onPresencePositionChange` | `(pos: PresencePosition) => void` | Callback | `handlePresencePositionChange` |
| `presenceLevel` | `PresenceLevel` | One-way | Internal state `presenceLevel` |
| `onPresenceLevelChange` | `(level: PresenceLevel) => void` | Callback | `handlePresenceLevelChange` |

## 9. Template Structure

```
div.tray-panel
  header
    h2 "52Hz"
  TimerStatus
  div.session-count  "Today's sessions: {todaySessions}" (`tray.session_count`)
  TimerControls
  SettingsForm
  button.quit-btn  "Quit app" (`tray.quit`)
```

## 10. Styles

| Selector | Property | Value |
|----------|----------|-------|
| `.tray-panel` | display | flex |
| `.tray-panel` | flex-direction | column |
| `.tray-panel` | height | 100% |
| `.tray-panel` | padding | 1rem |
| `.tray-panel` | gap | 0.8rem |
| `header h2` | font-size | 1rem |
| `header h2` | font-weight | 600 |
| `header h2` | text-align | center |
| `header h2` | color | var(--text-secondary) |
| `header h2` | letter-spacing | 0.1em |
| `header h2` | text-transform | uppercase |
| `.session-count` | font-size | 0.8rem |
| `.session-count` | text-align | center |
| `.session-count` | color | var(--text-secondary) |
| `.quit-btn` | margin-top | auto |
| `.quit-btn` | padding | 0.4rem |
| `.quit-btn` | font-size | 0.75rem |
| `.quit-btn` | border | none |
| `.quit-btn` | border-radius | 4px |
| `.quit-btn` | background | transparent |
| `.quit-btn` | color | var(--text-secondary) |
| `.quit-btn` | cursor | pointer |
| `.quit-btn` | transition | color 0.2s |
| `.quit-btn:hover` | color | var(--danger) |

Scoped CSS (component-scoped via Svelte's `<style>`).

## 11. Dependencies

### 11.1 Internal Modules

| Module | Imported items |
|--------|---------------|
| `../lib/timer` | `TimerState` (type), `remainingSecs`, `formatTime`, `getTimerState`, `togglePause`, `updateSettings`, `onTimerTick`, `quitApp`, `onPhaseChanged`, `resetTimer`, `getTodaySessions`, `setTrayIconVisible` |
| `../lib/settings-store` | `loadSettings`, `saveSettings`, `toTimerSettings`, `loadPauseMediaOnBreak`, `savePauseMediaOnBreak`, `loadHideTrayIcon`, `saveHideTrayIcon`, `loadTickVolume`, `saveTickVolume`, `loadPresencePosition`, `savePresencePosition`, `loadPresenceLevel`, `savePresenceLevel`, `PresencePosition` (type), `PresenceLevel` (type) |

### 11.2 Child Components

| Component | Path |
|-----------|------|
| `TimerStatus` | `./TimerStatus.svelte` |
| `TimerControls` | `./TimerControls.svelte` |
| `SettingsForm` | `./SettingsForm.svelte` |

### 11.3 External Packages (Including Indirect Dependencies)

| Package | Purpose |
|---------|---------|
| `svelte` | onMount, onDestroy lifecycle |
| `@tauri-apps/api/core` | invoke (via timer.ts) |
| `@tauri-apps/api/event` | listen (via timer.ts) |
| `@tauri-apps/plugin-store` | Settings persistence (via settings-store.ts) |
| `@tauri-apps/plugin-autostart` | macOS login auto-start management (direct import) |

## 12. Design Notes

1. **Initial value fallback**: The initial value of `phaseLabel` is `"Focus"` (English), but it is updated to the localized label by `handleTick` immediately after mount. There is a possibility of briefly seeing the English text on the first render.
2. **unlisten type cast**: The return value of `onTimerTick` is cast with `as unknown as () => void`. This is due to a type mismatch between Tauri's `listen` return type (`UnlistenFn`) and Svelte's expected type.
3. **Settings reflection order**: `handleSaveSettings` performs backend reflection (`updateSettings`) first, followed by store save (`saveSettings`). Even if backend reflection fails, the store save is still executed (no error handling).
4. **Backend reflection on settings load**: `loadSavedSettings` calls `updateSettings` after loading settings, immediately reflecting persisted settings to the backend timer.
5. **Separation of session count subscription and initialization**: `todaySessions` fetches its initial value with `getTodaySessions()` on mount, and subsequently re-fetches only within the `onPhaseChanged` callback. Using the phase change event instead of `timer-tick` avoids per-second calls.
6. **Multiple unlisten management**: `onDestroy` must call unlisten functions for both `timer-tick` and `phase-changed`. Failing to unsubscribe either one causes an event listener leak.
