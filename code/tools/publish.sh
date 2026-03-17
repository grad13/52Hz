#!/bin/bash
# meta: updated=2026-03-17 07:42 checked=-
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
  code/app/frontend/ \
  code/app/tauri/ \
  code/app/public/ \
  code/tools/ \
  code/app/index.html \
  code/app/package.json \
  code/app/package-lock.json \
  code/app/svelte.config.js \
  code/app/tsconfig.json \
  code/app/vite.config.ts \
  code/app/vitest.config.ts \
  code/app/CHANGELOG.md \
  tests/ \
  documents/spec/ \
  documents/cassette-spec.md \
  code/app/.face/ \
  .github/ \
  README.md \
  LICENSE

# 公開不要ファイルを除外
git reset HEAD -- '**/.DS_Store' '**/__pycache__/' 2>/dev/null || true
git checkout -- '**/.DS_Store' 2>/dev/null || true

# NOTE: .gitignore はブランチごとに個別管理（同期しない）
# NOTE: code/prototypes/ は非公開（同期しない）

if ! git diff --cached --quiet; then
  git commit -m "sync from local"
fi

git push origin main

# タグが local 専用コミットを指している場合、main のコミットに付け替えて push
# --points-at local だと HEAD のみ。--merged local --no-merged main で全対象を検出
retagged=()
for tag in $(git tag --merged local --no-merged main 2>/dev/null); do
  echo "Re-tagging $tag to main..."
  git tag -d "$tag"
  git tag "$tag" main
  retagged+=("$tag")
done

for tag in "${retagged[@]}"; do
  echo "Pushing tag $tag..."
  git push origin "$tag"
done

git checkout local
echo "Published successfully."
