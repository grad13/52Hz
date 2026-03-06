---
name: code-to-spec
description: コードから仕様ドキュメントを抽出・生成するスキル（#1）。
---

# Code to Spec

コードから仕様ドキュメントを抽出・生成するスキル（#1）。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: コードファイルを収集
3. **調査**: サブエージェント並列でcode↔spec対応を調査（1 file = 1 agent）
4. **生成**: サブエージェント並列でspecを生成（1 file = 1 agent）
5. **サマリー**: 統計・一覧を `.spec-from-code/summary.md` に出力
6. **ユーザー確認**: 概要提示 → 許可取得
7. **適用**: 許可後に `documents/spec/` へ配置
8. **クリーンアップ**: 作業ディレクトリ削除
9. **Post**: git commit & tag（完了記録）

## 概要

| 項目 | 内容 |
|------|------|
| 入力 | コードディレクトリ or `--from-refactor` |
| 出力 | `documents/.spec-from-code/` → 確認後 `documents/spec/` |
| 形式 | spec-v2.1（推奨）- Contract + State + Logic 同梱 + Source×Runtime |
| テンプレート | `templates/spec-v2.1.md`, `templates/summary.md` |

## 入力オプション

### オプションA: ディレクトリ指定

```
code-to-spec apps/ffffff/code/api
code-to-spec apps/ffffff/code/js/panels
```

指定ディレクトリ内のコードファイルを対象とする。

### オプションB: refactor-codeの結果を使用

```
code-to-spec --from-refactor
```

`code/.refactor/_summary.md` からファイルリストを取得。
refactor-codeで分析済みのファイルを優先的に処理できる。

## ワークフロー

### Step 0: Pre - git commit & tag（ロールバックポイント）

**スキル開始前に現在の状態を保存する。**

```bash
# 未コミットの変更があればコミット
git add -A
git diff --cached --quiet || git commit -m "pre: code-to-spec checkpoint"

# タグを作成（ロールバック用）
git tag code-to-spec-pre-$(date +%Y%m%d-%H%M%S)
```

問題発生時に `git reset --hard code-to-spec-pre-XXXXXXXX` でロールバック可能にする。

### Step 1: 対象選定

**ディレクトリ指定の場合:**
```bash
# 指定ディレクトリ内のコードファイルを収集
# 除外: _archive/, _prototypes/, _exemplars/, _tools/, データファイル
# 除外: テストファイル(*_tests.rs, *_test.rs, *.test.ts, *.spec.ts)
# 除外: HTMLテンプレート(*.html)
```

**--from-refactorの場合:**
```bash
# code/.refactor/_summary.md を読み込み
# must/ と should/ に記録されたファイルを対象リストに追加
# CSSファイルは除外（specの対象外）
# テストファイル(*_tests.rs, *_test.rs, *.test.ts, *.spec.ts)は除外
# HTMLテンプレート(*.html)は除外
```

### Step 2: spec調査（1ファイル = 1サブエージェント、並列実行）

各コードファイルについて:

1. **ファイル分類（調査前フィルタ）**
   - **ref 判定**（以下のいずれかに該当 → spec 不要、調査・生成をスキップ）:
     a. メタコメントに `ref=X` がある → 委譲先 X を記録
     b. 構造的 ref（メタコメント不要）:
        - エントリポイント: `main.rs`, `main.ts`（関数呼び出し1つのみ）
        - ビルドスクリプト: `build.rs`（tauri_build 委譲のみ）
        - 自動生成型定義: `vite-env.d.ts`
        - ツール設定: `vite.config.ts`, `vitest.config.ts`, `svelte.config.js`
     → 結果に `ref:{相対パス}→{委譲先}` と記録
   - 上記以外 → 通常の調査へ進む

2. **コードを読み込み**
   - 公開インターフェース（関数、クラス、API）を特定
   - 主要な責務を把握

3. **対応specを検索**
   - `documents/spec/` 内で関連するspecを検索
   - ファイル名、内容のキーワードでマッチング

4. **カバレッジ判定**
   - **covered**: specが存在し、主要機能がカバーされている
   - **partial**: specは存在するが、一部機能が未記載（不足内容を明記）
   - **missing**: 対応するspecが存在しない
   - **ref**: spec 不要（薄いラッパー、委譲先を記録）

5. **結果出力**
   - 調査結果を `.spec-from-code/analysis/` に出力

### Step 3: spec生成（missing + partial両方を処理）

**重要: missingだけでなく、partialも処理する。**

#### missingの場合（新規spec）

1. 形式を決定（判定基準参照）
2. 完全なspecを生成
3. `documents/.spec-from-code/generated/{カテゴリ}/{ファイル名}.md` に出力

#### partialの場合（追記用spec）

1. 既存specの形式を確認
2. 不足している仕様のみを抽出・生成
3. `documents/.spec-from-code/generated/{カテゴリ}/{ファイル名}-supplement.md` に出力
4. フロントマターに `Type: Supplement` と `Target: {既存specパス}` を記載

### Step 4: サマリー生成

`documents/.spec-from-code/summary.md` を生成:
- 統計（covered/partial/missing件数、生成したspec件数）
- 新規spec一覧
- 追記用spec一覧
- 推奨アクション

### Step 5: ユーザーへの概要提示と許可取得

**重要: 適用前に必ずユーザーの許可を得る。**

#### 提示内容

ユーザーに以下の概要を提示する:

1. **統計サマリー**
   - 対象ファイル数、covered/partial/missing件数
   - 生成したspec件数（新規/追記）

2. **新規specの説明**（各ファイルについて）
   - 対象コード
   - 形式（Contract/Gherkin/StateMachine/DecisionTable）
   - 主要な内容（States, Transitions, API等）

3. **追記用specの説明**（各ファイルについて）
   - 対象コード → ターゲットspec
   - 追加される内容の概要

#### 許可取得

- ユーザーが「更新しましょう」等の許可を出すまで待機
- 質問があれば回答する
- 修正要望があれば対応する

### Step 6: 適用（許可後）

ユーザーの許可を得てから:

1. **新規spec**: `.spec-from-code/generated/` から `documents/spec/` にコピー
2. **追記用spec**: 既存specにマージ（末尾に追記 or セクション統合）
3. 既存specの `更新日:` を更新

### Step 7: クリーンアップ（許可後）

適用完了後、ユーザーの許可を得てから:

1. `.spec-from-code/` ディレクトリを削除

### Step 8: Post - git commit & tag（完了記録）

**スキル完了後に結果をコミット & タグ付け。**

```bash
git add -A
git commit -m "code-to-spec: add {N} specs from code"
git tag code-to-spec-post-$(date +%Y%m%d-%H%M%S)
```

## 出力構造

```
documents/
├── .spec-from-code/
│   ├── summary.md            # 全体サマリー（次工程へのバトン）
│   ├── analysis/             # 調査結果
│   │   ├── js-panel-assemble-index.md
│   │   └── api-v3-works.md
│   └── generated/            # 生成されたspec（確認待ち）
│       ├── journal/
│       │   ├── editor.md             # 新規spec
│       │   └── storage.md            # 新規spec
│       ├── panel/
│       │   └── panel-assemble-supplement.md  # 追記用spec
│       └── schedule/
│           └── store-contract.md     # 追記用spec
└── spec/                     # 既存spec + 適用後のspec
    └── ...
```

## 形式

**推奨: spec-v2.1形式**（`templates/spec-v2.1.md`）

spec-v2.1は以下を1ファイルに同梱:
- **Meta**: Source × Runtime テーブル（各ソースの言語・モジュール形式を明示）
- **Contract**: 型定義（TypeScript）
- **State**: 状態遷移（Mermaid）
- **Logic**: 決定表（Markdown Table）
- **Side Effects**: 副作用（Integration用）

コードの特徴に関わらず、v2.1形式で統一することでテスト生成が容易になる。

### Runtime 判定

spec 生成時、各 Source の Runtime を以下のルールで判定する:

1. 拡張子 `.rs` → `Rust`
2. 拡張子 `.ts`/`.svelte` で `import`/`export` 文あり → `JS-ESM`

判定結果を spec の Meta > Source テーブルに記載する。

### メタコメント形式

コードファイルの1行目に記載。code-to-spec がファイル分類に使用する。

| 言語 | 形式 | 例 |
|------|------|-----|
| Rust `.rs` | `// meta: key=value ...` | `// meta: ref=lib` |
| TypeScript `.ts` | `// meta: key=value ...` | `// meta: ref=App` |
| Svelte `.svelte` | `<!-- meta: key=value ... -->` | `<!-- meta: ref=TrayPanel -->` |

キー: `ref=X`（委譲先）
- 構造的 ref に該当するファイルはメタコメント不要

## summary.md フォーマット

テンプレート: `templates/summary.md`

```markdown
# Code to Spec Summary

実行日: {YYYY-MM-DD}

## Spec一覧

| Spec | Format | Source | Check | Action |
|------|--------|--------|-------|--------|
| {specファイルパス} | {Format} | {ソースコードパス} | {covered/partial/missing/ref} | {Action} |
```

**目的**: 次工程（spec-to-tests）へのバトン渡し

| 列 | 説明 |
|----|------|
| Spec | documents/spec/ からの相対パス |
| Format | Contract / StateMachine / Gherkin / DecisionTable |
| Source | code/ からの相対パス |
| Check | covered(完備), partial(一部不足), missing(無し), ref(委譲のみ) |
| Action | -(何もしない), generated(新規), updated(追記) |

次工程は `Action=generated` または `Action=updated` の行を処理対象とする。

## サブエージェントへの指示

### 調査フェーズ

```
対象コードの調査タスク

## 入力
- コードファイル: {絶対パス}
- specディレクトリ: {documents/spec/の絶対パス}

## 手順
1. コードをReadで読み込む
2. 公開インターフェースを特定:
   - 関数名、クラス名
   - APIエンドポイント（PHPの場合）
   - エクスポートされるオブジェクト（JSの場合）
3. documents/spec/ 内で関連specを検索:
   - Grepでファイル名・関数名を検索
   - 見つかったファイルを読んで確認
4. カバレッジを判定:
   - covered: 主要機能がすべてspec化済み
   - partial: 一部未記載（何が不足か明記）
   - missing: 対応spec無し

## 出力
Writeで以下に出力:
{documents_dir}/.spec-from-code/analysis/{相対パス}.md

フォーマット:
```markdown
# {相対パス}

## コード概要
- 行数: X
- 主要機能: ...
- 公開インターフェース: ...

## 対応spec
- ファイル: ...（あれば）
- カバレッジ: covered/partial/missing

## 不足している仕様（partialの場合）
- 機能A: 未記載
- 機能B: 未記載

## 推奨形式（missingの場合）
Contract / Gherkin / StateMachine / DecisionTable
```

## 戻り値
`covered:{相対パス}` または `partial:{相対パス}:{不足内容}` または `missing:{相対パス}`
`ref:{相対パス}→{委譲先}`
```

### 生成フェーズ（新規spec用）

```
spec生成タスク

## 入力
- コードファイル: {絶対パス}
- テンプレート: templates/spec-v2.1.md

## 手順
1. コードをReadで読み込む
2. テンプレート（templates/spec-v2.1.md）を参照してspecを生成:
   - 0. Meta: Source×Runtimeテーブル, Related, Test Type
   - 1. Contract: 型定義
   - 2. State: 状態遷移（あれば）
   - 3. Logic: 決定表
   - 4. Side Effects: 副作用（あれば）
3. フロントマターを含める:
   ---
   作成日: {今日}
   更新日: {今日}
   確認日: -
   使用終了日: -
   Format: spec-v2.1
   Source: {コードファイルの相対パス}
   ---

## 出力
Writeで以下に出力:
{documents_dir}/.spec-from-code/generated/{spec相対パス}.md

## 戻り値
`generated:{spec相対パス}`
```

### 生成フェーズ（追記用spec用）

```
spec追記生成タスク

## 入力
- コードファイル: {絶対パス}
- 既存spec: {既存specの絶対パス}
- 不足内容: {調査フェーズで特定した不足内容}
- 形式: 既存specの形式に合わせる

## 手順
1. コードをReadで読み込む
2. 既存specをReadで読み込み、形式を確認
3. 不足している仕様のみを抽出・生成
4. フロントマターを含める:
   ---
   作成日: {今日}
   更新日: {今日}
   確認日: -
   使用終了日: -
   Type: Supplement
   Target: {既存specの相対パス}
   Source: {コードファイルの相対パス}
   ---
5. 内容は「## 追記内容」として、既存specに追加すべきセクションを記述

## 出力
Writeで以下に出力:
{documents_dir}/.spec-from-code/generated/{カテゴリ}/{ファイル名}-supplement.md

## 戻り値
`generated:{spec相対パス}`
```

## 除外対象

- `_archive/` - アーカイブ済みコード
- `_prototypes/` - プロトタイプ
- `_exemplars/` - リファレンス
- `_tools/` - ツールスクリプト
- CSSファイル - specの対象外
- データファイル（`.json`, `.csv`, `.sql`, `.sqlite`等）

## 注意事項

- **確認フロー必須**: 生成specは `.spec-from-code/generated/` に出力し、ユーザーに概要を提示して許可を得てから適用（Step 5参照）
- **partial必須処理**: partial判定のファイルも追記用specを生成する（missingだけでなく）
- **形式統一**: 生成するspecはspec-v2.1形式（`templates/spec-v2.1.md`）で統一
- **出所明記**: 生成specのフロントマターに `Source:` でコードファイルを記録
- **追記用の区別**: 追記用specは `-supplement.md` サフィックス + `Type: Supplement` で区別
