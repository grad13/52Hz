# CLAUDE.md (RestRun)

RestRun アプリ固有のガイダンス。

## プロジェクト概要

RestRun - macOS 休憩リマインダーアプリ（DeskRest クローン）。
Tauri v2 + Svelte 5 + Rust で構成。

## ディレクトリ構成

```
27-RestRun/
├── CLAUDE.md
├── project-meta.json
├── documents/
│   ├── spec/           # 振る舞い仕様
│   ├── plan/           # 計画
│   ├── reference/      # 参照資料
│   ├── reports/        # レポート
│   └── archive/        # アーカイブ
├── src/                # フロントエンド（Svelte 5）
│   ├── App.svelte
│   ├── lib/
│   │   ├── Settings.svelte
│   │   ├── BreakOverlay.svelte
│   │   └── timer.ts
│   └── app.css
├── src-tauri/          # バックエンド（Rust / Tauri v2）
│   ├── src/
│   │   ├── lib.rs      # Tauriセットアップ、コマンド、オーバーレイ管理
│   │   ├── timer.rs    # タイマーロジック（ユニットテスト内蔵）
│   │   └── main.rs
│   ├── tests/
│   │   └── app_lifecycle.rs  # 統合テスト
│   ├── Cargo.toml
│   └── tauri.conf.json
└── .claude/
    └── skills/         # スキル定義
```

## ゴミを置かない

上記構造に含まれないファイルを作成しない。

## ビルド・テスト

### ビルド

```bash
cd src-tauri && cargo build
```

### ユニットテスト（Rust）

```bash
cd src-tauri && cargo test --lib
```

タイマーロジックのテストは `src-tauri/src/timer.rs` にインラインで定義（69テスト）。

### 統合テスト

```bash
cd src-tauri && cargo test --test app_lifecycle -- --test-threads=1
```

プロセス起動 → stderr ログ検査のパターン。Vite dev server の有無で分岐。

### フロントエンド型チェック

```bash
npx svelte-check
```

## テスト方針

- ユニットテスト: `#[cfg(test)]` インライン（timer.rs）
- 統合テスト: `src-tauri/tests/app_lifecycle.rs`（プロセス起動 + stderr ログアサーション）
- フロントエンド: `svelte-check` による型検査

## macOS 固有の実装

- `ActivationPolicy::Accessory` — Dock 非表示
- `NSApplicationPresentationOptions` — 休憩中の UI ロック（HideDock, HideMenuBar, DisableProcessSwitching, DisableHideApplication, DisableForceQuit）
- `activateIgnoringOtherApps` — トレイポップアップのフォーカス管理
- `NSWindow::setLevel(1000)` + `NSWindowCollectionBehavior` — オーバーレイの全画面表示

## タイマーアーキテクチャ

- `advance()` → `emit timer-tick` → `try_transition()` の3段階
- `<` 比較でフェーズは正確に `duration` tick で完了
- `apply_settings()` は現在のフェーズの duration も即時更新
