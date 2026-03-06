#!/bin/bash
# .refactorの結果から対象ファイルを特定し、checkedを更新するスクリプト
# 使用例: ./update-checked.sh /path/to/code
#
# .refactor/_summary.md または must/, should/ のファイルから対象を特定

CODE_DIR="${1:-.}"
REFACTOR_DIR="$CODE_DIR/.refactor"
TODAY=$(date +%Y-%m-%d)

if [[ ! -d "$REFACTOR_DIR" ]]; then
    echo "ERROR: .refactor directory not found: $REFACTOR_DIR"
    exit 1
fi

update_file() {
    local file="$1"
    local ext="${file##*.}"

    if [[ ! -f "$file" ]]; then
        echo "SKIP (not found): $file"
        return
    fi

    case "$ext" in
        rs|ts|svelte)
            # // meta: created=... updated=... checked=YYYY-MM-DD
            sed -i '' "s|checked=[0-9-]*|checked=$TODAY|" "$file"
            sed -i '' "s|checked=never|checked=$TODAY|" "$file"
            ;;
        md)
            # <!-- meta: created=... updated=... checked=YYYY-MM-DD -->
            sed -i '' "s|checked=[0-9-]*|checked=$TODAY|" "$file"
            sed -i '' "s|checked=never|checked=$TODAY|" "$file"
            ;;
    esac

    echo "UPDATED: $file"
}

# must/ と should/ からファイルパスを抽出
for md_file in "$REFACTOR_DIR"/must/*.md "$REFACTOR_DIR"/should/*.md; do
    [[ ! -f "$md_file" ]] && continue

    # File: から相対パスを取得
    rel_path=$(grep -m1 "^File:" "$md_file" | sed 's/File: *//')
    [[ -z "$rel_path" ]] && continue

    target_file="$CODE_DIR/$rel_path"
    update_file "$target_file"
done

# _summary.md の clean セクションからも取得
if [[ -f "$REFACTOR_DIR/_summary.md" ]]; then
    in_clean=false
    while IFS= read -r line; do
        if [[ "$line" =~ ^##.*clean ]]; then
            in_clean=true
            continue
        fi
        if [[ "$line" =~ ^## ]] && $in_clean; then
            break
        fi
        if $in_clean && [[ "$line" =~ ^- ]]; then
            rel_path=$(echo "$line" | sed 's/^- *//')
            target_file="$CODE_DIR/$rel_path"
            update_file "$target_file"
        fi
    done < "$REFACTOR_DIR/_summary.md"
fi

echo ""
echo "Complete!"
