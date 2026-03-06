# CLAUDE.md (52Hz)

52Hz アプリ固有のガイダンス。

## プロジェクト概要

52Hz - macOS 休憩リマインダーアプリ（DeskRest クローン）。
Tauri v2 + Svelte 5 + Rust で構成。

## ディレクトリ構成

```
27-52Hz/
├── .claude/                # Claude Code 設定・スキル
├── CLAUDE.md
├── project-meta.json
│
├── code/                   # アプリを動かすためのコード
│   ├── frontend/           # フロントエンド（Svelte 5）
│   │   ├── App.svelte
│   │   ├── components/     # UI コンポーネント
│   │   │   ├── Settings.svelte
│   │   │   └── BreakOverlay.svelte
│   │   └── lib/            # 純粋ロジック
│   │       └── timer.ts
│   └── tauri/              # バックエンド（Rust / Tauri v2）
│       ├── src/
│       │   ├── lib.rs      # Tauriセットアップ、コマンド、オーバーレイ管理
│       │   ├── timer.rs    # タイマーロジック（ユニットテスト内蔵）
│       │   └── main.rs
│       ├── Cargo.toml
│       └── tauri.conf.json
│
├── tests/                  # テスト関係
│   ├── integration/
│   │   └── app_lifecycle.rs  # 統合テスト（正本）
│   └── test-run.sh
│
├── _documents/              # ドキュメント
│   ├── spec/ plan/ reference/ reports/ archive/
│
└── externals/              # ユーザが渡す資料
```

## ゴミを置かない

上記構造に含まれないファイルを作成しない。

## externals/ のルール

- `externals/` はユーザが配置する資料置き場。Claude が直接改変しない。
- 参照・加工する場合は `code/_prototype/` 等にコピーしてから作業すること。
- Claude が生成した試作物・中間ファイルは `code/_prototype/` に保存する。

## ビルド・テスト

### ビルド

```bash
cd code/tauri && cargo build
```

### ユニットテスト（Rust）

```bash
cd code/tauri && cargo test --lib
```

タイマーロジックのテストは `code/tauri/src/timer.rs` にインラインで定義（69テスト）。

### 統合テスト

```bash
cd code/tauri && cargo test --test app_lifecycle -- --test-threads=1
```

プロセス起動 → stderr ログ検査のパターン。Vite dev server の有無で分岐。
`code/tauri/tests/app_lifecycle.rs` は `tests/integration/app_lifecycle.rs` へのシンボリックリンク。

### フロントエンド型チェック

```bash
cd code && npx svelte-check
```

## テスト方針

- ユニットテスト: `#[cfg(test)]` インライン（timer.rs）
- 統合テスト: `tests/integration/app_lifecycle.rs`（プロセス起動 + stderr ログアサーション）
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
