---
name: refactor-tests
description: テストファイルのリファクタリング候補を分析するスキル。問題の本質と理想の状態を明確化する。
---

# Refactor Tests

テストファイルのリファクタリング候補を分析するスキル。問題の本質と理想の状態を明確化する。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: `code/tauri/src/` の `#[cfg(test)]` インラインテスト + `tests/integration/` を収集
3. **対象決定**: 500行超 → must候補、残りは行数降順、合計20件
4. **分析**: サブエージェント順次でファイル分析（1 file = 1 agent、最大20件）
5. **サマリー**: `tests/.refactor-tests/summary.md` に出力
6. **Post**: git commit & tag（完了記録）

※ 分析のみ。実際のリファクタリングは別途実施。
※ テスト↔コードの整合性は `tests-to-code`、テスト↔specの整合性は `tests-to-spec` で対処。

## 原則

- **500行超は必ず分割対象**: テストファイルも1ファイル = 1責務
- **cargo test 統一**: テスト実行は `cd code/tauri && cargo test --lib`
- **use 文統一**: `use crate::`/`use super::` の使い方をプロジェクト規約に従う
- **cargo check ゲート**: 分割後に `cargo check` で新規エラーがないことを確認
- **trait + テスト実装**: モックは trait を定義しテスト用の実装を作成。手書き部分モック禁止
- **テスト内部のみ分析**: ソースコードやspecは読まない。テスト↔コード/specの整合性は `tests-to-code` / `tests-to-spec` の責務

## 判定基準

| 判定 | ID | 条件 | 説明 |
|------|----|------|------|
| **must** | M2 | 500行超 | テストファイルが500行を超えている |
| **should** | S2 | （廃止） | — |
| **should** | S3 | use文不整合 | `use crate::`/`use super::` の使い方が不統一 |
| **should** | S6 | 責務混在 | 複数モジュールを1テストファイルでテスト |
| **should** | S7 | 手書き部分モック | trait + テスト実装ではなく手書きの部分構造体でモック |
| **should** | S8 | cargo check エラー | `cargo check` でエラーが出る |

## 実行モード

### フォアグラウンド（デフォルト）
```
refactor-tests
```
進捗がリアルタイムで表示される。

### バックグラウンド
```
refactor-tests --background
```
処理全体を1つのbackgroundタスクとして実行。完了通知が届く。
結果は `tests/.refactor-tests/summary.md` を参照。

実装方法:
```
Task:
- subagent_type: "general-purpose"
- run_in_background: true
- prompt: "refactor-testsスキルを実行"
```

## ワークフロー

### Step 0: Pre - git commit & tag（ロールバックポイント）

**スキル開始前に現在の状態を保存する。**

```bash
# 未コミットの変更があればコミット
git add -A
git diff --cached --quiet || git commit -m "pre: refactor-tests checkpoint"

# タグを作成（ロールバック用）
git tag refactor-tests-pre-$(date +%Y%m%d-%H%M%S)
```

問題発生時に `git reset --hard refactor-tests-pre-XXXXXXXX` でロールバック可能にする。

### Step 1: 対象選定

`code/tauri/src/` 配下の `*.rs` ファイル（`#[cfg(test)]` を含むもの）と `tests/integration/` 配下の `*.rs` を Grep/Glob で収集する。

**除外対象**:
- `_archive/` 配下
- `target/` 配下
- `.refactor-tests/` 配下（作業ディレクトリ）

### Step 2: 対象決定

1. 500行超を `must` としてリストアップ
2. 残りを行数降順でソート
3. must + 行数上位 = 合計20ファイル（mustが20以上なら全て）

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

全サブエージェント完了後、`tests/.refactor-tests/summary.md` を生成。

### Step 5: Post - git commit & tag（完了記録）

**スキル完了後に結果をコミット & タグ付け。**

```bash
git add -A
git commit -m "refactor-tests: analyze {N} files"
git tag refactor-tests-post-$(date +%Y%m%d-%H%M%S)
```

## 出力構造

```
tests/.refactor-tests/
├── must/                         # must判定（必須対象）
│   └── tauri/src/timer.rs.md          # 元のパス構造を保持
├── should/                       # should判定（推奨対象）
│   └── tauri/src/lib.rs.md
└── summary.md
```

**重要: 出力パスは元のcode/以下の構造を保持する。**
- 例: `code/tauri/src/timer.rs` → `tests/.refactor-tests/must/tauri/src/timer.rs.md`

## 分析結果ファイルのフォーマット

```markdown
---
File: {相対パス}
Lines: {行数}
Judgment: {must/should}
Issues: [{問題IDリスト}]
---

# {ファイル名}

## 問題点

### 1. [{問題ID}] {問題タイトル}

**現状**: {何が起きているか、該当行}
**本質**: {なぜそれが問題なのか}
**あるべき姿**: {どうあるべきか}
```

## サブエージェントへの指示

**1ファイル = 1サブエージェント**。各サブエージェントは**同一の分析プロセス**を実行する。

### プロンプトテンプレート

```
テストファイル分析タスク

対象: {絶対パス}

## 手順
1. Readでテストファイル読込
2. 判定（テストファイルのみで判定。ソースコード・specは読まない）:
   - 500行超（M2）→ must
   - S2, S3, S6, S7, S8 → should
   - 該当なし → clean

## 出力（must/shouldの場合のみ）
Writeツールで以下に出力:
- mustの場合: {tests_dir}/.refactor-tests/must/{相対パス}.md
- shouldの場合: {tests_dir}/.refactor-tests/should/{相対パス}.md

**重要**: 必ず `/must/` または `/should/` ディレクトリの下に出力すること。

## 戻り値
`must:{相対パス}` または `should:{相対パス}` または `clean:{相対パス}`
```

### 分析フロー（全ファイル共通）

**重要: Bashツール・スクリプト実行は禁止。Read/Write/Grepツールのみ使用。**

```
1. Readツールでテストファイルを読み込む
   （ソースコード・specは読まない。テストファイルのみで分析する）
2. 基本情報を確認:
   - 行数: Readツールの出力行番号から確認
   - テストケース数: `#[test] fn` を数える
   - use文の確認（`use crate::`/`use super::` の整理状況、参照先モジュール数）
3. 問題点チェック:
   - M2: 500行超？
   - S2: （廃止）
   - S3: `use crate::`/`use super::` の使い方が不統一か？
   - S6: 複数モジュールを1ファイルでテストしていないか？（use文から判定）
   - S7: trait + テスト実装ではなく手書きの部分構造体でモックしているか？
   - S8: `cargo check` エラーがあるか？（コード内の型記述から推定）
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
- テストファイルの修正（分析のみ）

**重要: 問題があるファイルは、必ずWriteツールでファイルに書き出すこと。**

### 出力

各サブエージェントは分析結果を `tests/.refactor-tests/` に書き出し、**最小限の戻り値**を返す:

```
must:tauri/src/timer.rs
should:tauri/src/lib.rs
clean:integration/app_lifecycle.rs
```

**パス構造保持**: 出力ファイルは元のcode/以下またはtests/以下の構造を維持する。
- 入力: `code/tauri/src/timer.rs`
- 出力: `tests/.refactor-tests/must/tauri/src/timer.rs.md`

**コンテキスト節約**: 詳細は全てファイルに書き出す。戻り値は判定と相対パスのみ。

## 除外対象

- `_archive/` - アーカイブ済みテスト
- `target/` - ビルド出力
- `.refactor-tests/` - 作業ディレクトリ

## 次工程

`refactor-tests` はテスト内部品質のみを分析する。検出した問題（M2, S2, S3, S6, S7, S8）は計画書を作成して手動で対処する。

テスト↔コード/spec の整合性チェックは別スキルの責務:

| 整合性 | スキル | 検出する問題 |
|--------|--------|-------------|
| test ↔ code | `tests-to-code` | 自己充足テスト（C1）、IF不整合（C2） |
| test ↔ spec | `tests-to-spec` | spec欠落（Case A）、乖離（B）、不足（C）、マッピング不明（D）、構造不一致（E） |

## 注意事項

- **バッチ化禁止**: 1ファイル = 1サブエージェント
- **同期実行**: run_in_backgroundは使用しない（ハングする問題あり）
- **20ファイル制限**: 1回の実行で最大20ファイル（mustが多ければそれ以上）
- **許可不要**: サブエージェントは `mode="bypassPermissions"` で実行し、ユーザーに許可を求めない
- **ファイル出力必須**: must/shouldの場合、必ず `tests/.refactor-tests/` にWriteツールでファイルを書き出す
- **パス構造保持**: 出力ファイルは元のtests/以下のディレクトリ構造を維持する
- **出力先明示**: プロンプトで絶対パスを明示（サブエージェントが誤った場所に出力する問題を防ぐ）
- **Bash禁止**: Read/Write/Grepツールのみ使用
- **分析のみ**: テストファイルの修正は行わない
