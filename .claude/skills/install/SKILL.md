---
name: install
description: コード変更を即座にビルドして /Applications にインストールし、アプリを起動する。
---

# install — ビルド＆インストール

コード変更を即座にビルドして /Applications にインストールし、アプリを起動する。
バージョン更新・CHANGELOG・タグ作成は行わない（それらは `/deploy` で行う）。

## 手順

```bash
# 1. 実行中の 52Hz を停止
pkill -f "52Hz" 2>/dev/null; sleep 1

# 2. .app のみビルド（DMG は作らない）
cd code && npx tauri build --bundles app

# 3. /Applications にインストール（cp -R は上書きするので rm 不要）
cp -R code/tauri/target/release/bundle/macos/52Hz.app /Applications/

# 4. 起動
open /Applications/52Hz.app
```

## 注意

- ビルド失敗時はインストール・起動をスキップし、エラーを報告する
- DMG は不要。`--bundles app` で .app のみ生成する
