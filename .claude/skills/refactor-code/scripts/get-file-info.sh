#!/bin/bash
# ファイル情報を取得し、チェック対象を選定するスクリプト
# 使用例: ./get-file-info.sh /path/to/code
#
# 出力形式: lines|updated|checked|priority|file
# priority: -1 = unchecked(∞), 0+ = days since last check

CODE_DIR="${1:-.}"
TODAY=$(date +%Y-%m-%d)

calc_priority() {
    local updated="$1"
    local checked="$2"

    if [[ "$checked" == "-" || "$checked" == "never" || -z "$checked" ]]; then
        echo "-1"  # 未チェック = 最優先
        return
    fi

    # 日数差を計算 (macOS)
    local updated_sec=$(date -j -f "%Y-%m-%d" "$updated" +%s 2>/dev/null || echo 0)
    local checked_sec=$(date -j -f "%Y-%m-%d" "$checked" +%s 2>/dev/null || echo 0)

    if [[ $updated_sec -eq 0 || $checked_sec -eq 0 ]]; then
        echo "-1"
        return
    fi

    local diff=$(( (updated_sec - checked_sec) / 86400 ))
    echo "$diff"
}

get_file_info() {
    local file="$1"
    local lines=$(wc -l < "$file" | tr -d " ")
    # metadata format: // meta: created=... updated=... checked=...
    # Rust (.rs), TypeScript (.ts), Svelte (.svelte) で共通
    local updated=$(grep -m1 "meta:.*updated=" "$file" 2>/dev/null | sed 's/.*updated=\([0-9-]*\).*/\1/' | tr -d " ")
    local checked=$(grep -m1 "meta:.*checked=" "$file" 2>/dev/null | sed 's/.*checked=\([0-9-]*\).*/\1/' | tr -d " ")

    [[ -z "$updated" ]] && updated="-"
    [[ -z "$checked" ]] && checked="-"

    local priority=$(calc_priority "$updated" "$checked")

    echo "$lines|$updated|$checked|$priority|$file"
}

# ファイル収集と情報取得
find "$CODE_DIR" -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.svelte" \) \
    ! -path "*/_archive/*" \
    ! -path "*/_prototype/*" \
    ! -path "*/node_modules/*" \
    ! -path "*/target/*" \
    ! -path "*/.build/*" \
    2>/dev/null | while read file; do
        get_file_info "$file"
    done
