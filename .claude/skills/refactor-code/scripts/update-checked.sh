#!/bin/bash
# .refactorの結果から対象ファイルを特定し、checkedを更新するスクリプト
# 使用例: ./update-checked.sh /path/to/code
#
# .refactor/summary.md または must/, should/ のファイルから対象を特定

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

    # ref= のみのファイル（委譲先）はスキップ
    if grep -q "^// meta: ref=" "$file" 2>/dev/null || grep -q "^<!-- meta: ref=" "$file" 2>/dev/null; then
        echo "SKIP (ref only): $file"
        return
    fi

    case "$ext" in
        rs|ts|svelte)
            if grep -q "^// meta:.*checked=" "$file"; then
                sed -i '' "/^\/\/ meta:/s|checked=[0-9-]*|checked=$TODAY|" "$file"
                sed -i '' "/^\/\/ meta:/s|checked=never|checked=$TODAY|" "$file"
                echo "UPDATED: $file"
            else
                sed -i '' "1i\\
// meta: checked=$TODAY
" "$file"
                echo "ADDED: $file"
            fi
            ;;
        md)
            if grep -q "^<!-- meta:.*checked=" "$file"; then
                sed -i '' "/^<!-- meta:/s|checked=[0-9-]*|checked=$TODAY|" "$file"
                sed -i '' "/^<!-- meta:/s|checked=never|checked=$TODAY|" "$file"
                echo "UPDATED: $file"
            else
                sed -i '' "1i\\
<!-- meta: checked=$TODAY -->
" "$file"
                echo "ADDED: $file"
            fi
            ;;
    esac
}

# must/ と should/ からファイルパスを再帰的に抽出
find "$REFACTOR_DIR"/must "$REFACTOR_DIR"/should -name "*.md" 2>/dev/null | while IFS= read -r md_file; do
    [[ ! -f "$md_file" ]] && continue

    # File: から相対パスを取得、なければファイルパスから復元
    rel_path=$(grep -m1 "^File:" "$md_file" | sed 's/File: *//')
    if [[ -z "$rel_path" ]]; then
        # must/tauri/src/lib.rs.md → tauri/src/lib.rs
        rel_path="${md_file#$REFACTOR_DIR/must/}"
        rel_path="${rel_path#$REFACTOR_DIR/should/}"
        rel_path="${rel_path%.md}"
    fi
    [[ -z "$rel_path" ]] && continue

    target_file="$CODE_DIR/$rel_path"
    update_file "$target_file"
done

# summary.md の clean セクションからも取得
if [[ -f "$REFACTOR_DIR/summary.md" ]]; then
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
            rel_path=$(echo "$line" | sed 's/^- *//; s/ (.*//')
            target_file="$CODE_DIR/$rel_path"
            update_file "$target_file"
        fi
    done < "$REFACTOR_DIR/summary.md"
fi

echo ""
echo "Complete!"
