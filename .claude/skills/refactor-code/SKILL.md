---
name: refactor-code
description: コードのリファクタリング候補を分析する。「refactor-code <path>」で使用。500行超ファイルの分割、責務分離を行う。
---

# Code Refactor

コードのリファクタリング候補を分析するスキル。問題の本質と理想の状態を明確化する。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: スクリプトでファイル情報取得 → must/優先度でソート
3. **分析**: サブエージェント順次でファイル分析（1 file = 1 agent、最大20件）
4. **サマリー**: must/should一覧を `.refactor/summary.md` に出力
5. **checked更新**: 分析済みファイルの `// meta:` コメント更新
6. **Post**: git commit & tag（完了記録）

※ 分析のみ。実際のリファクタリングは別途実施。

## 原則

- **500行超は必ず分割対象**: データファイル（JSON, CSV等）は除く
- **No Fallback**: fallbackは極力用いない
- **責任分担の明確化**: 1ファイル = 1責務
- **モジュール・trait で責務分離**: 分割時、Rust の mod/trait 抽出や TS のモジュール分離で依存を明確化

## ファイルメタデータ

各ソースファイル先頭にメタデータコメント:
- Rust: `// meta: created=YYYY-MM-DD updated=YYYY-MM-DD checked=YYYY-MM-DD`
- TS/Svelte: `// meta: created=YYYY-MM-DD updated=YYYY-MM-DD checked=YYYY-MM-DD`
- `checked` が `-` の場合は未チェック（例: `checked=-`）

## スクリプト

```
.claude/skills/refactor-code/scripts/
├── get-file-info.sh    # ファイル情報取得・対象選定
└── update-checked.sh   # checked一括更新（// meta: コメント）
```

## 実行モード

### フォアグラウンド（デフォルト）
```
refactor-code /path/to/code
```
進捗がリアルタイムで表示される。

### バックグラウンド
```
refactor-code /path/to/code --background
```
処理全体を1つのbackgroundタスクとして実行。完了通知が届く。
結果は `.refactor/summary.md` を参照。

実装方法:
```
Task:
- subagent_type: "general-purpose"
- run_in_background: true
- prompt: "refactor-codeスキルを実行: /path/to/code"
```

## ワークフロー

### Step 0: Pre - git commit & tag（ロールバックポイント）

**スキル開始前に現在の状態を保存する。**

```bash
# 未コミットの変更があればコミット
git add -A
git diff --cached --quiet || git commit -m "pre: refactor-code checkpoint"

# タグを作成（ロールバック用）
git tag refactor-code-pre-$(date +%Y%m%d-%H%M%S)
```

問題発生時に `git reset --hard refactor-code-pre-XXXXXXXX` でロールバック可能にする。

### Step 1: 対象選定

```bash
{skill_dir}/scripts/get-file-info.sh {code_dir}
```

### Step 2: 対象決定

1. 500行超を `must` としてリストアップ
2. 残りを priority 順にソート（同率はシャッフル）
3. must + 優先度上位 = 合計20ファイル（mustが20以上なら全て）

### Step 3: 分析

**1ファイル = 1サブエージェント**（バッチ化禁止）

**同期実行（順次処理）**: 20ファイルを順番に処理する。

```
各Task設定:
- subagent_type: "general-purpose"
- mode: "bypassPermissions"
# run_in_background は指定しない（非同期だとハングする問題あり）
```

**注意**: `run_in_background: true` を指定するとハングする問題があるため、同期実行を使用する。

### Step 4: サマリー生成

全サブエージェント完了後、`code/.refactor/summary.md` を生成

### Step 5: checked更新

```bash
{skill_dir}/scripts/update-checked.sh {code_dir}
```

### Step 6: Post - git commit & tag（完了記録）

**スキル完了後に結果をコミット & タグ付け。**

```bash
git add -A
git commit -m "refactor-code: analyze {N} files"
git tag refactor-code-post-$(date +%Y%m%d-%H%M%S)
```

## 判定基準

| 判定 | 条件 | 出力先 |
|------|------|--------|
| **must** | 500行超 | `must/` |
| **should** | 責務混在、fallback検出 | `should/` |
| **clean** | 上記に該当せず | `summary.md` のみ |

## 出力構造

```
code/.refactor/
├── must/                                  # 500行超（必須対象）
│   └── tauri/src/lib.rs.md               # 元のパス構造を保持
├── should/                                # 責務混在、fallback（推奨対象）
│   └── frontend/components/Settings.svelte.md
└── summary.md
```

**重要: 出力パスは元のcode/以下の構造を保持する。**
- 例: `code/tauri/src/lib.rs` → `code/.refactor/should/tauri/src/lib.rs.md`

## 分析結果ファイルのフォーマット

```markdown
---
File: {相対パス}
Lines: {行数}
Judgment: {must/should}
Issues: [{問題点リスト}]
---

# {ファイル名}

## 問題点

### 1. {問題タイトル}

**現状**: {何が起きているか、該当行}
**本質**: {なぜそれが問題なのか}
**あるべき姿**: {どうあるべきか}
```

## サブエージェントへの指示

**1ファイル = 1サブエージェント**。各サブエージェントは**同一の分析プロセス**を実行する。

### プロンプトテンプレート

```
ファイル分析タスク

対象: {絶対パス}

## 手順
1. Readでファイル読込
2. 判定:
   - 500行超 → must
   - 責務混在またはfallback → should
   - 該当なし → clean

## 出力（must/shouldの場合のみ）
Writeツールで以下に出力:
- mustの場合: {code_dir}/.refactor/must/{相対パス}.md
- shouldの場合: {code_dir}/.refactor/should/{相対パス}.md

**重要**: 必ず `/must/` または `/should/` ディレクトリの下に出力すること。

## 戻り値
`must:{相対パス}` または `should:{相対パス}` または `clean:{相対パス}`
```

### 分析フロー（全ファイル共通）

**重要: Bashツール・スクリプト実行は禁止。ReadツールとWriteツールのみ使用。**

```
1. Readツールでファイルを読み込む
2. 基本情報を確認:
   - 行数: Readツールの出力行番号から確認
   - 関数/クラス数: コードを見て数える
3. 問題点チェック:
   - 500行超？ → must
   - 責務混在？ → should
   - fallbackパターン？ → should
4. 問題があれば:
   - 問題点を詳細分析
   - **必ずWriteツールでファイルに書き出す**（must/ または should/）
5. 問題なければ:
   - 「clean」としてファイル名を返す（サマリー用）
```

**禁止事項**:
- Bashツールの使用
- スクリプトの作成・実行
- 外部コマンドの実行

**重要: 問題があるファイルは、必ずWriteツールでファイルに書き出すこと。**

### 出力

各サブエージェントは分析結果を `code/.refactor/` に書き出し、**最小限の戻り値**を返す:

```
must:tauri/src/lib.rs
should:frontend/components/Settings.svelte
clean:frontend/App.svelte
```

**パス構造保持**: 出力ファイルは元のcode/以下の構造を維持する。
- 入力: `code/tauri/src/lib.rs`
- 出力: `code/.refactor/should/tauri/src/lib.rs.md`

**コンテキスト節約**: 詳細は全てファイルに書き出す。戻り値は判定と相対パスのみ。

## 除外対象

- `_archive/` - アーカイブ済みコード
- `_prototypes/` - 試行錯誤のプロトタイプ
- `_exemplars/` - 仕様理解のリファレンス
- `_tools/` - ツールスクリプト
- データファイル（`.json`, `.csv`, `.sql`, `.sqlite`, `.toml`（設定のみ）, `.html` 等）

## 注意事項

- **バッチ化禁止**: 1ファイル = 1サブエージェント
- **同期実行**: run_in_backgroundは使用しない（ハングする問題あり）
- **20ファイル制限**: 1回の実行で最大20ファイル（mustが多ければそれ以上）
- **許可不要**: サブエージェントは `mode="bypassPermissions"` で実行し、ユーザーに許可を求めない
- **ファイル出力必須**: must/shouldの場合、必ず `code/.refactor/` にWriteツールでファイルを書き出す
- **パス構造保持**: 出力ファイルは元のcode/以下のディレクトリ構造を維持する
- **出力先明示**: プロンプトで絶対パスを明示（サブエージェントが誤った場所に出力する問題を防ぐ）
- **Bash禁止**: ReadツールとWriteツールのみ使用
