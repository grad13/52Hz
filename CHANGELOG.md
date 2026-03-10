# Changelog

## [Unreleased]

### Added
- i18n support with English as default language and Japanese translation
- Language selector (EN/JA toggle) in tray panel

## [0.6.2] - 2026-03-06

### Changed
- Migrated presence data from JSON to SQLite (chat.db)
- Rewrote presence.rs to use direct rusqlite queries

### Fixed
- Fixed bug where toast remained in foreground after focus-done dismiss in back mode

## [0.6.1] - 2026-03-06

### Added
- In back mode, clicking a toast raises it to foreground (second click dismisses)
- Auto-raise to foreground when focus-done popup appears
- Timestamp display (top-right) on all popups

### Changed
- Auto-return to back mode when all items are dismissed

## [0.6.0] - 2026-03-06

### Added
- Tick sound volume slider (changed from on/off to continuous 0-1 adjustment)
- Presence display: selectable position from 4 corners
- Presence display: front/back layer toggle
- Presence display: dedicated options section in menu panel
- Reverse toast stack order when displayed at bottom (newest at bottom)
- /install skill (quick build without version bump)

### Changed
- Expanded menu panel height from 420 to 480 (for new section)

### Fixed
- Updated existing tests to match new UI structure, props, and mocks (all 225 tests passing)

## [0.5.5] - 2026-03-04

### Added
- Browser media toggle: pause/resume YouTube in Firefox during breaks via keystroke

### Fixed
- Focus-done toast no longer dismissed when presence toasts are turned off
- Toast window position: clear macOS menu bar (margin_top 40px)

## [0.5.4] - 2026-03-04

### Changed
- Focus-done popup restyled to match toast card format (276px, compact dark card)
- Focus-done popup window background transparent (floating card)
- Presence scheduler: time-aware persona filtering with rewritten 1,305 messages

### Fixed
- Unnecessary `unsafe` block warning in toast transparency code

## [0.5.3] - 2026-03-04

### Fixed
- Toast window transparency: delay _setDrawsBackground until WKWebView is loaded

## [0.5.2] - 2026-03-04

### Changed
- Toast window background fully transparent (cards float on screen)
- Presence scheduler emits ~1 message/min in a steady loop
- Hover poll now hit-tests against actual timer element, not entire panel

### Fixed
- Button focus outline (blue border) on toast cards
- Timer hover/pause triggered by non-timer areas of tray panel

### Removed
- test_presence_toast debug command

## [0.5.0] - 2026-03-04

### Added
- Presence toast system — 50 virtual personas with realistic activity schedules
- Toast notifications slide in at top-right, max 10 stacked, each displayed up to 3 minutes
- Click-to-dismiss toasts (with macOS first-click activation workaround)
- "Presence" toggle in settings to enable/disable presence toasts
- Toast window: always-on-top, visible on all Spaces, rounded corners

### Fixed
- Toast window capabilities (event listening, show/hide, resize permissions)
- Hidden titlebar buttons on toast window

## [0.4.2] - 2026-03-04

### Added
- Tick sound option — plays wall clock tick every second while timer is running
- npm scripts: `install-app` and `deploy` for build + install workflow

### Changed
- Tray panel window height 500 → 560 to accommodate new toggles

## [0.4.1] - 2026-03-04

### Added
- Hide tray icon option — toggle in Settings to show only timer text in menu bar
- Spec, test coverage for the new option

## [0.4.0] - 2026-03-03

### Changed
- App icon to newshuilette whale + eye design (traced from newshuilette.png via potrace)
- Tray icon display height 15pt → 18pt
- Archive old icon variants and prototype files to code/_archive/

### Added
- SVG source files in icons/svg-source/ (app-icon-v1, newshuilette_with_eye_bg)
- externals/ rules to CLAUDE.md

### Removed
- Unused android/ios icon assets

## [0.2.1] - 2026-02-28

### Changed
- Stop tracking Cargo.lock (gitignore)

## [0.2.0] - 2026-02-28

### Changed
- Restructure directories to code/frontend + code/tauri pattern
- Extract timer tests to timer_tests.rs
- Extract Tauri commands to commands.rs
- Extract overlay management to overlay.rs
- Extract tray menu to tray.rs
- Move spawn_timer and SharedTimerState from timer.rs to lib.rs
- Split Settings.svelte into TrayPanel + child components
- Extract settings persistence to settings-store.ts

### Added
- 13 specs from code
- 8 test files (100 cases) from spec
- 7 additional tests generated from code
- Headless test mode plan

### Fixed
- 7 test-to-code fixes (A1/A2/A4 categories)
- 3 specs updated from test feedback

## [0.1.0] - 2026-02-26

Initial version. Basic break reminder with timer, tray menu, and overlay.
