---
Created: 2026-02-26
Updated: 2026-03-08

Verified: -
Deprecated: -
Format: spec-v2.1
Source: frontend/lib/settings-store.ts
---

# settings-store spec

Runtime: JS-ESM (TypeScript)

## 0. Meta

| Item | Value |
|------|-------|
| Module name | `settings-store` |
| Path | `frontend/lib/settings-store.ts` |
| Responsibility | Unit conversion between display and internal settings values, and persistence via Tauri plugin-store |
| Dependencies (external) | `@tauri-apps/plugin-store` |
| Dependencies (internal) | `./timer` (type `TimerSettings` only) |
| State | None (stateless utility) |
| Side effects | File I/O (via Tauri plugin-store) |
| Internal helper | `getStore()` -- centralizes `load("settings.json", { autoSave: true })` |

## 1. Contract

### 1.1 Type Definitions

#### DisplaySettings (export interface)

Holds settings values for UI display. Units are in human-friendly form.

| Field | Type | Unit | Constraint |
|-------|------|------|------------|
| `focusMinutes` | `number` | minutes | > 0 |
| `shortBreakMinutes` | `number` | minutes | > 0 |
| `longBreakMinutes` | `number` | minutes | > 0 |
| `shortBreaksBeforeLong` | `number` | times | >= 1 (integer) |

#### TimerSettings (import from ./timer)

Settings values compatible with the Rust backend. All fields in seconds (short_breaks_before_long is a count).

| Field | Type | Unit |
|-------|------|------|
| `focus_duration_secs` | `number` | seconds |
| `short_break_duration_secs` | `number` | seconds |
| `long_break_duration_secs` | `number` | seconds |
| `short_breaks_before_long` | `number` | times |

#### PresencePosition (export type)

Union type representing the display position of presence toasts.

```
"top-right" | "top-left" | "bottom-right" | "bottom-left"
```

#### PresenceLevel (export type)

Union type representing the display level (Z-order) of presence toasts.

```
"always-front" | "dynamic" | "always-back"
```

| Value | NSWindowLevel | Behavior |
|-------|--------------|----------|
| `"always-front"` | 25 | Stays in front even when other windows are clicked |
| `"dynamic"` | 0 | Normal window. Goes behind when other windows are clicked |
| `"always-back"` | -1 | Always behind other windows |

#### PresenceLikeIcon (export type)

Union type representing the "like" icon type for presence toasts.

```
"heart" | "star" | "none"
```

### 1.2 Functions

#### getStore (Internal helper, not exported)

```
() => Promise<Store>
```

- Async function (not exported)
- Helper centralizing `load("settings.json", { autoSave: true })`
- All load/save functions obtain the store via this helper
- Type cast `as Parameters<typeof load>[1]` is also centralized here

#### toTimerSettings

```
(d: DisplaySettings) => TimerSettings
```

- Synchronous, pure function
- Converts DisplaySettings to TimerSettings

#### toDisplaySettings

```
(s: TimerSettings) => DisplaySettings
```

- Synchronous, pure function
- Converts TimerSettings to DisplaySettings

#### loadSettings

```
() => Promise<DisplaySettings | null>
```

- Async function
- Reads settings from plugin-store and returns them as DisplaySettings
- Returns `null` when settings are unsaved (all keys null) or on error

#### saveSettings

```
(d: DisplaySettings) => Promise<void>
```

- Async function
- Writes DisplaySettings to plugin-store
- On error, the exception propagates to the caller

#### loadPauseMediaOnBreak

```
() => Promise<boolean>
```

- Async function
- Reads the `pause_media_on_break` key from plugin-store and returns it as boolean
- Returns `false` when the key is unset (null)
- Returns `false` on error (swallowed)

#### savePauseMediaOnBreak

```
(enabled: boolean) => Promise<void>
```

- Async function
- Writes the `pause_media_on_break` key to plugin-store
- On error, the exception propagates to the caller

#### loadHideTrayIcon

```
() => Promise<boolean>
```

- Async function
- Reads the `hide_tray_icon` key from plugin-store and returns it as boolean
- Returns `false` when the key is unset (null)
- Returns `false` on error (swallowed)

#### saveHideTrayIcon

```
(enabled: boolean) => Promise<void>
```

- Async function
- Writes the `hide_tray_icon` key to plugin-store
- On error, the exception propagates to the caller

#### loadTickVolume

```
() => Promise<number>
```

- Async function
- Reads the `tick_sound` key from plugin-store and returns it as number
- Has backward compatibility logic (was stored as boolean in older versions)
- Returns `0` when the key is unset (null) or `false`
- Returns `0` on error (swallowed)

#### saveTickVolume

```
(volume: number) => Promise<void>
```

- Async function
- Writes the `tick_sound` key to plugin-store as a number
- On error, the exception propagates to the caller

#### loadPresenceToast

```
() => Promise<boolean>
```

- Async function
- Reads the `presence_toast` key from plugin-store and returns it as boolean
- Returns `true` when the key is unset (null) (default ON)
- Returns `true` on error (swallowed)

#### savePresenceToast

```
(enabled: boolean) => Promise<void>
```

- Async function
- Writes the `presence_toast` key to plugin-store
- On error, the exception propagates to the caller

#### loadPresencePosition

```
() => Promise<PresencePosition>
```

- Async function
- Reads the `presence_position` key from plugin-store and returns it as PresencePosition
- Returns `"top-right"` when the key is unset (null)
- Returns `"top-right"` on error (swallowed)

#### savePresencePosition

```
(pos: PresencePosition) => Promise<void>
```

- Async function
- Writes the `presence_position` key to plugin-store
- On error, the exception propagates to the caller

#### loadPresenceLevel

```
() => Promise<PresenceLevel>
```

- Async function
- Reads the `presence_level` key from plugin-store and returns it as PresenceLevel
- Backward compatibility: converts legacy value `"front"` -> `"always-front"`, `"back"` -> `"always-back"`
- Returns `"dynamic"` when the key is unset (null) or has an unknown value
- Returns `"dynamic"` on error (swallowed)

#### savePresenceLevel

```
(level: PresenceLevel) => Promise<void>
```

- Async function
- Writes the `presence_level` key to plugin-store
- On error, the exception propagates to the caller

#### loadPresenceMaxToasts

```
() => Promise<number>
```

- Async function
- Reads the `presence_max_toasts` key from plugin-store and returns it as number
- Returns `4` when the key is unset (null)
- Returns `4` on error (swallowed)

#### savePresenceMaxToasts

```
(n: number) => Promise<void>
```

- Async function
- Writes the `presence_max_toasts` key to plugin-store
- On error, the exception propagates to the caller

#### loadPresenceShowIcon

```
() => Promise<boolean>
```

- Async function
- Reads the `presence_show_icon` key from plugin-store and returns it as boolean
- Returns `true` when the key is unset (null) (default ON)
- Returns `true` on error (swallowed)

#### savePresenceShowIcon

```
(v: boolean) => Promise<void>
```

- Async function
- Writes the `presence_show_icon` key to plugin-store
- On error, the exception propagates to the caller

#### loadPresenceLikeIcon

```
() => Promise<PresenceLikeIcon>
```

- Async function
- Reads the `presence_like_icon` key from plugin-store and returns it as PresenceLikeIcon
- Returns `"heart"` when the value is not one of the known values (`"heart"`, `"star"`, `"none"`)
- Returns `"heart"` when the key is unset (null)
- Returns `"heart"` on error (swallowed)

#### savePresenceLikeIcon

```
(v: PresenceLikeIcon) => Promise<void>
```

- Async function
- Writes the `presence_like_icon` key to plugin-store
- On error, the exception propagates to the caller

## 2. State

This module holds no internal state.

Persistence target state:

| Store file | Key | Type | Corresponding field |
|-----------|-----|------|---------------------|
| `settings.json` | `focus_minutes` | `number` | `DisplaySettings.focusMinutes` |
| `settings.json` | `short_break_minutes` | `number` | `DisplaySettings.shortBreakMinutes` |
| `settings.json` | `long_break_minutes` | `number` | `DisplaySettings.longBreakMinutes` |
| `settings.json` | `short_breaks_before_long` | `number` | `DisplaySettings.shortBreaksBeforeLong` |
| `settings.json` | `pause_media_on_break` | `boolean` | Independent setting (not included in DisplaySettings) |
| `settings.json` | `hide_tray_icon` | `boolean` | Independent setting (not included in DisplaySettings) |
| `settings.json` | `tick_sound` | `number` (legacy: `boolean`) | Independent setting (tick volume 0-1) |
| `settings.json` | `presence_toast` | `boolean` | Independent setting (presence toast display) |
| `settings.json` | `presence_position` | `PresencePosition` | Independent setting (toast display position) |
| `settings.json` | `presence_level` | `PresenceLevel` | Independent setting (toast display level) |
| `settings.json` | `presence_max_toasts` | `number` | Independent setting (max toast count) |
| `settings.json` | `presence_show_icon` | `boolean` | Independent setting (show user icon on toast) |
| `settings.json` | `presence_like_icon` | `PresenceLikeIcon` | Independent setting (like icon type) |

## 3. Logic

### 3.1 Unit Conversion

Conversion table between `DisplaySettings` and `TimerSettings`:

| DisplaySettings | Conversion | TimerSettings |
|----------------|-----------|---------------|
| `focusMinutes` | `* 60` | `focus_duration_secs` |
| `shortBreakMinutes` | `* 60` | `short_break_duration_secs` |
| `longBreakMinutes` | `* 60` | `long_break_duration_secs` |
| `shortBreaksBeforeLong` | as-is | `short_breaks_before_long` |

Reverse conversion (`toDisplaySettings`) recovers `focusMinutes`, `shortBreakMinutes`, `longBreakMinutes` by `/ 60`.

Invariant: `toDisplaySettings(toTimerSettings(d))` matches the original `d`
(assuming no floating-point rounding errors).

### 3.2 loadSettings Flow

```
1. getStore()
2. Fetch 4 keys individually with store.get<number>()
3. if all keys == null:
     return null          // First launch / settings unsaved
4. else:
     return {
       each field: fetched value ?? default value
     }
5. catch:
     return null          // Error swallowed
```

Default values table:

| Field | Default value |
|-------|--------------|
| `focusMinutes` | 20 |
| `shortBreakMinutes` | 1 |
| `longBreakMinutes` | 3 |
| `shortBreaksBeforeLong` | 3 |

### 3.3 saveSettings Flow

```
1. getStore()
2. Write 4 fields individually with store.set()
3. autoSave: true automatically persists to disk
```

### 3.4 loadPauseMediaOnBreak Flow

```
1. getStore()
2. store.get<boolean>("pause_media_on_break")
3. return val ?? false
4. catch:
     return false          // Error swallowed
```

### 3.5 savePauseMediaOnBreak Flow

```
1. getStore()
2. store.set("pause_media_on_break", enabled)
3. autoSave: true automatically persists to disk
```

### 3.6 loadHideTrayIcon Flow

```
1. getStore()
2. store.get<boolean>("hide_tray_icon")
3. return val ?? false
4. catch:
     return false          // Error swallowed
```

### 3.7 saveHideTrayIcon Flow

```
1. getStore()
2. store.set("hide_tray_icon", enabled)
3. autoSave: true automatically persists to disk
```

### 3.8 loadTickVolume Flow

```
1. getStore()
2. store.get<number | boolean>("tick_sound")
3. Backward compatibility branching:
     if val === true:  return 0.5    // Legacy boolean format -> mid volume
     if val === false:  return 0     // Legacy boolean format -> mute
     if val == null:   return 0      // Unset -> mute
     else:             return val    // Return number as-is
4. catch:
     return 0          // Error swallowed
```

### 3.9 saveTickVolume Flow

```
1. getStore()
2. store.set("tick_sound", volume)
3. autoSave: true automatically persists to disk
```

### 3.10 loadPresenceToast Flow

```
1. getStore()
2. store.get<boolean>("presence_toast")
3. return val ?? true               // Default ON
4. catch:
     return true          // Error swallowed
```

### 3.11 savePresenceToast Flow

```
1. getStore()
2. store.set("presence_toast", enabled)
3. autoSave: true automatically persists to disk
```

### 3.12 loadPresencePosition Flow

```
1. getStore()
2. store.get<PresencePosition>("presence_position")
3. return val ?? "top-right"
4. catch:
     return "top-right"   // Error swallowed
```

### 3.13 savePresencePosition Flow

```
1. getStore()
2. store.set("presence_position", pos)
3. autoSave: true automatically persists to disk
```

### 3.14 loadPresenceLevel Flow

```
1. getStore()
2. store.get<string>("presence_level")
3. Migration:
     if val === "front":  return "always-front"   // Legacy -> new
     if val === "back":   return "always-back"    // Legacy -> new
     if val in ["always-front", "dynamic", "always-back"]:  return val
     else:                return "dynamic"         // Unset or unknown value
4. catch:
     return "dynamic"   // Error swallowed
```

### 3.15 savePresenceLevel Flow

```
1. getStore()
2. store.set("presence_level", level)
3. autoSave: true automatically persists to disk
```

### 3.16 loadPresenceMaxToasts Flow

```
1. getStore()
2. store.get<number>("presence_max_toasts")
3. return val ?? 4
4. catch:
     return 4          // Error swallowed
```

### 3.17 savePresenceMaxToasts Flow

```
1. getStore()
2. store.set("presence_max_toasts", n)
3. autoSave: true automatically persists to disk
```

### 3.18 loadPresenceShowIcon Flow

```
1. getStore()
2. store.get<boolean>("presence_show_icon")
3. return val ?? true               // Default ON
4. catch:
     return true          // Error swallowed
```

### 3.19 savePresenceShowIcon Flow

```
1. getStore()
2. store.set("presence_show_icon", v)
3. autoSave: true automatically persists to disk
```

### 3.20 loadPresenceLikeIcon Flow

```
1. getStore()
2. store.get<string>("presence_like_icon")
3. Validation:
     if val in ["heart", "star", "none"]:  return val
     else:                return "heart"    // Unset or unknown value
4. catch:
     return "heart"   // Error swallowed
```

### 3.21 savePresenceLikeIcon Flow

```
1. getStore()
2. store.set("presence_like_icon", v)
3. autoSave: true automatically persists to disk
```

## 4. Side Effects (Tauri invoke)

This module does not use Tauri invoke (IPC commands).
Instead, it performs file I/O through the `@tauri-apps/plugin-store` plugin.

| Operation | API | File | Notes |
|-----------|-----|------|-------|
| Store load | `getStore()` -> `load("settings.json", { autoSave: true })` | `$APPDATA/settings.json` | All load/save functions call via `getStore()` |
| Value read | `store.get<number>(key)` | - | Called 4 times in loadSettings |
| Value read | `store.get<boolean>("pause_media_on_break")` | - | Called 1 time in loadPauseMediaOnBreak |
| Value write | `store.set(key, value)` | - | Called 4 times in saveSettings. Auto-persisted via autoSave |
| Value write | `store.set("pause_media_on_break", boolean)` | - | Called 1 time in savePauseMediaOnBreak. Auto-persisted via autoSave |
| Value read | `store.get<boolean>("hide_tray_icon")` | - | Called 1 time in loadHideTrayIcon |
| Value write | `store.set("hide_tray_icon", boolean)` | - | Called 1 time in saveHideTrayIcon. Auto-persisted via autoSave |
| Value read | `store.get<number \| boolean>("tick_sound")` | - | Called 1 time in loadTickVolume |
| Value write | `store.set("tick_sound", number)` | - | Called 1 time in saveTickVolume. Auto-persisted via autoSave |
| Value read | `store.get<boolean>("presence_toast")` | - | Called 1 time in loadPresenceToast |
| Value write | `store.set("presence_toast", boolean)` | - | Called 1 time in savePresenceToast. Auto-persisted via autoSave |
| Value read | `store.get<PresencePosition>("presence_position")` | - | Called 1 time in loadPresencePosition |
| Value write | `store.set("presence_position", PresencePosition)` | - | Called 1 time in savePresencePosition. Auto-persisted via autoSave |
| Value read | `store.get<PresenceLevel>("presence_level")` | - | Called 1 time in loadPresenceLevel |
| Value write | `store.set("presence_level", PresenceLevel)` | - | Called 1 time in savePresenceLevel. Auto-persisted via autoSave |
| Value read | `store.get<number>("presence_max_toasts")` | - | Called 1 time in loadPresenceMaxToasts |
| Value write | `store.set("presence_max_toasts", number)` | - | Called 1 time in savePresenceMaxToasts. Auto-persisted via autoSave |
| Value read | `store.get<boolean>("presence_show_icon")` | - | Called 1 time in loadPresenceShowIcon |
| Value write | `store.set("presence_show_icon", boolean)` | - | Called 1 time in savePresenceShowIcon. Auto-persisted via autoSave |
| Value read | `store.get<string>("presence_like_icon")` | - | Called 1 time in loadPresenceLikeIcon |
| Value write | `store.set("presence_like_icon", PresenceLikeIcon)` | - | Called 1 time in savePresenceLikeIcon. Auto-persisted via autoSave |

## 5. Notes

### 5.1 Design Characteristics

- **Stateless**: Holds no module-level variables at all. Obtains the store via the `getStore()` helper every time.
- **Bidirectional conversion**: `toTimerSettings` / `toDisplaySettings` are symmetrically defined, absorbing the unit difference between the UI layer and the Rust layer.
- **Partial save recovery**: loadSettings complements missing keys with default values even when only some keys exist.

### 5.2 Asymmetric Error Handling

- `loadSettings`: Uses try-catch to convert exceptions to `null` (swallowed).
- `saveSettings`: No try-catch. Exceptions propagate to the caller.

This asymmetry is considered intentional. load may encounter cases where the store doesn't exist at startup, but save failures are critical errors that should be surfaced to the user.

### 5.3 tick_sound Backward Compatibility

The `tick_sound` key was stored as `boolean` in older versions.
`loadTickVolume` reads with a `number | boolean` union type, converting boolean values:
`true` -> `0.5`, `false` -> `0`. `saveTickVolume` always writes as `number`,
so once re-saved, subsequent reads will be in number format.

### 5.4 Default Value Variations

Default values for load functions vary by the nature of the setting:
- `false` as default: pauseMediaOnBreak, hideTrayIcon (opt-in features)
- `0` as default: tickVolume (mute by default)
- `true` as default: presenceToast (default ON)
- `"top-right"` as default: presencePosition
- `"dynamic"` as default: presenceLevel
- `4` as default: presenceMaxToasts (max toast count)
- `true` as default: presenceShowIcon (default ON)
- `"heart"` as default: presenceLikeIcon

### 5.5 presence_level Backward Compatibility

The `presence_level` key was stored as `"front"` / `"back"` (2 values) in older versions.
`loadPresenceLevel` reads as `string` and converts legacy values to new values:
- `"front"` -> `"always-front"`
- `"back"` -> `"always-back"`
- Unset -> `"dynamic"` (new default)

`savePresenceLevel` always writes new values, so once re-saved, new values will be read subsequently.

### 5.6 Type Cast Presence

A type cast `as Parameters<typeof load>[1]` is centralized in the `getStore()` helper for the second argument of `load()`.
This is presumably a workaround for type incompatibilities across plugin-store versions.
