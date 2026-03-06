---
name: deploy
description: ローカルビルド＆インストール（バージョン更新 + タグ付き）
---

# deploy — ビルド＆インストール

macOS デスクトップアプリをローカルでビルドし、/Applications にインストールする。
バージョン更新、CHANGELOG 更新、タグ作成を含む。

## 手順

### Step 1: 状態確認

```bash
git status --short
git tag --sort=-v:refname | head -5
```

未コミット変更がある場合はユーザーに報告し、先にコミットするか確認する。

### Step 2: bump 種別確認

ユーザーに patch / minor / major を確認し、新バージョンを算出する。

### Step 3: バージョン更新

プロジェクト固有の箇所を更新する:

| プロジェクト | ファイル | 箇所 |
|-------------|---------|------|
| 52Hz | `code/tauri/tauri.conf.json` | `"version"` |
| 52Hz | `code/tauri/Cargo.toml` | `version` |
| 52Hz | `code/package.json` | `"version"` |

### Step 4: CHANGELOG 更新

`_documents/CHANGELOG.md` に新エントリを追加する。

フォーマット:
```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added / Changed / Fixed / Removed
- 変更内容
```

変更内容は `git log --oneline <前回タグ>..HEAD` で確認する。

### Step 5: コミット

```bash
git add <更新したファイル> _documents/CHANGELOG.md
git commit -m "chore: bump version to vX.Y.Z"
```

### Step 6: ビルド＆インストール

```bash
cd code && npm run tauri build
```

ビルド成功後、自動で /Applications にインストールする:

```bash
# 既存の 52Hz.app があれば削除
rm -rf /Applications/52Hz.app
# ビルド成果物をコピー
cp -R code/tauri/target/release/bundle/macos/52Hz.app /Applications/
```

### Step 6.5: ビルドキャッシュ削除

```bash
cd code/tauri && cargo clean
```

デプロイ完了後にビルドキャッシュを削除し、ディスク容量を回収する（target/ は約10G）。
次回ビルド時にフルビルドが必要になるが、deploy の頻度は低いため許容範囲。

### Step 7: タグ作成（ビルド成功時のみ）

```bash
git tag vX.Y.Z
```

**ビルド失敗時はタグを作成しない。** コミットは残り、修正後に再度 deploy する。

## バージョン規約

- フォーマット: `vX.Y.Z`（semver）
- **patch (Z)**: バグ修正、表示修正
- **minor (Y)**: 機能追加、既存機能の拡張
- **major (X)**: アーキテクチャ変更、破壊的変更

## 注意

- 3ファイルのバージョンと CHANGELOG のバージョンが不一致の場合、deploy 実行時に正しいバージョンに揃える
- publish（GitHub リリース）は別スキル `/publish` で行う
