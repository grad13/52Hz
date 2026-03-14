# Changelog

## [0.7.2] - 2026-03-14

### Changed
- `.face/news.json` の date を Unix timestamp に変換（Hub 新規約対応）
- `.face/app.json` に `thumbnail` フィールドを追加
- GitHub Actions CI ワークフロー追加

## [0.7.1] - 2026-03-13

### Added
- `.face/` ディレクトリ（Hub プロジェクト掲載用メタデータ）

### Fixed
- トーストウィンドウの透明領域がクリックスルーされるように修正
- トレイパネルの高さ同期に ResizeObserver を使用

## [0.7.0] - 2026-03-11

### Added
- カセットセレクターUI（TrayPanel内、アクセントカラー付きカード）
- カセット切替・フォルダオープン機能（IPC: listCassettes, switchCassette, openCassetteFolder）
- i18n: EN/JA 対応（カセット関連テキスト）
- GitHub README にアプリアイコン・トレイアイコンを追加

### Changed
- i18n 基盤整備（English デフォルト、言語セレクター）

### Fixed
- トレイアイコン非表示オプションのバグ修正（AtomicBool で状態管理）
- トースト z-order バグ修正（dynamic モード対応）
- FocusDonePopup 削除（Toast ベースに統合済み）
- 保存済みロケールの起動時適用修正
- テストの unhandled rejection 解消（全テストファイルのモック整備）

## [0.6.2] - 2026-03-06

### Changed
- presence データを JSON から SQLite (chat.db) に移行
- presence.rs を rusqlite 直接クエリに書き換え

### Fixed
- 背面モードでフォーカス完了 dismiss 後にトーストが前面に残るバグを修正

## [0.6.1] - 2026-03-06

### Added
- 背面モード時、トーストクリックで前面に浮上（2回目クリックで消去）
- フォーカス完了ポップアップ出現時に自動で前面に浮上
- 全ポップアップにタイムスタンプ表示（右上）

### Changed
- 全アイテム消去時に自動で背面に復帰

## [0.6.0] - 2026-03-06

### Added
- Tick音ボリュームスライダー（on/off → 0〜1の連続調整）
- みんなの存在：表示位置を4方向から選択可能（↖↗↙↘）
- みんなの存在：前面/背面表示の切替
- みんなの存在：専用オプションセクションをメニューパネルに追加
- 下部表示時のトースト逆順スタック（新しいものが下に）
- /install スキル（バージョン更新なしのクイックビルド）

### Changed
- メニューパネル高さを420→480に拡大（新セクション対応）

### Fixed
- 既存テストを新しいUI構造・props・モックに合わせて修正（225テスト全パス）

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
- "みんなの存在" toggle in settings to enable/disable presence toasts
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
