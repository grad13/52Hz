#!/bin/bash
# テスト用スクリプト: tauri devを経由せず直接バイナリを実行
set -e

cd "$(dirname "$0")/../code"

echo "=== Step 1: Vite dev server 起動 ==="
npx vite --port 1420 &
VITE_PID=$!
sleep 2

echo ""
echo "=== Step 2: バイナリ直接実行 (Ctrl+C で終了) ==="
echo "=== もしすぐにプロンプトが戻ったら、バイナリが落ちています ==="
echo ""
./tauri/target/debug/restrun
BINARY_EXIT=$?

echo ""
echo "=== バイナリが終了しました (exit code: $BINARY_EXIT) ==="

kill $VITE_PID 2>/dev/null
