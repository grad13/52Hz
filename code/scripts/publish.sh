#!/bin/bash
set -e

current=$(git branch --show-current)
if [ "$current" != "local" ]; then
  echo "Error: must be on local branch"; exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Error: uncommitted changes exist."; exit 1
fi

git checkout main

# local から公開ファイルのみ同期（ホワイトリスト方式）
git checkout local -- \
  code/frontend/ \
  code/tauri/ \
  code/public/ \
  code/scripts/ \
  code/index.html \
  code/package.json \
  code/package-lock.json \
  code/svelte.config.js \
  code/tsconfig.json \
  code/vite.config.ts \
  code/vitest.config.ts \
  code/CHANGELOG.md \
  tests/ \
  documents/spec/ \
  documents/cassette-spec.md \
  code/.face/ \
  .github/ \
  README.md \
  LICENSE

# 公開不要ファイルを除外
git reset HEAD -- '**/.DS_Store' '**/__pycache__/' 2>/dev/null || true
git checkout -- '**/.DS_Store' 2>/dev/null || true

# NOTE: .gitignore はブランチごとに個別管理（同期しない）
# NOTE: code/_prototype/, code/_archive/ は非公開（同期しない）

if ! git diff --cached --quiet; then
  git commit -m "sync from local"
fi

git push origin main
git checkout local
echo "Published successfully."
