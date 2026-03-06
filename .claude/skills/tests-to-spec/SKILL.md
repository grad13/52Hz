---
name: tests-to-spec
description: テストを起点に test ↔ spec 間の整合性問題を発見・対処するスキル。
---

# Tests to Spec

テストを起点に test ↔ spec 間の整合性問題を発見・対処するスキル。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: テストファイルを収集
3. **分析**: サブエージェント並列で問題を特定・分類（1 test = 1 agent）
4. **生成**: サブエージェント並列で spec を生成/修正（1 test = 1 agent）
5. **サマリー**: 統計・一覧を `documents/.spec-from-tests/summary.md` に出力
6. **ユーザー確認**: 概要提示 → 許可取得
7. **適用**: 許可後に `documents/spec/` へ配置 + テストの `Spec:` ヘッダー追加
8. **クリーンアップ**: 作業ディレクトリ削除
9. **Post**: git commit & tag（完了記録）

## 概要

| 項目 | 内容 |
|------|------|
| 入力 | テストディレクトリ or `--from-refactor-tests` |
| 出力 | `documents/.spec-from-tests/` → 確認後 `documents/spec/` |
| 形式 | spec-v2.1（Contract + State + Logic 同梱 + Source×Runtime） |
| 一次情報源 | テストファイル |
| 補助情報源 | ソースコード（Meta・Contract の型情報補完用） |

## 問題分類（Case A〜E）

テストを中心に、test ↔ spec 間の整合性問題を5つに分類する:

| Case | 問題 | 説明 |
|------|------|------|
| A | spec欠落 | テストはあるが対応する spec がない |
| B | 乖離 | テストと spec の内容が食い違っている |
| C | spec不足 | テストが検証する範囲を spec がカバーしきれていない |
| D | マッピング不明 | どのテストがどの spec に対応するか不明確 |
| E | 構造の不一致 | spec のセクション構造とテストの構造が対応しない |

**「その他」は設けない。** A〜E に分類できない問題が見つかった場合はユーザーに報告・相談する。

### Case ごとの対処方針

| Case | 対処 |
|------|------|
| A | spec-v2.1 を新規生成 |
| B | spec の該当箇所を修正（テストを正とする） |
| C | spec に不足分を追記（supplement） |
| D | テストに `Spec:` ヘッダー追加 + 必要なら spec 側にも Source 追記 |
| E | spec の構造をテスト構造に合わせて再構成 |

## 入力オプション

### オプションA: ディレクトリ指定

```
tests-to-spec tests/integration
tests-to-spec code/tauri/src
```

指定ディレクトリ内のテストファイル（`*.rs`（`#[cfg(test)]` インライン or 統合テスト））を対象とする。

### オプションB: refactor-testsの結果を使用

```
tests-to-spec --from-refactor-tests
```

`tests/.refactor-tests/summary.md` から S1（spec未参照）判定のファイルリストを取得。
should/ ディレクトリ内の分析ファイルも追加コンテキストとして活用する。

## ワークフロー

### Step 0: Pre - git commit & tag（ロールバックポイント）

**スキル開始前に現在の状態を保存する。**

```bash
git add -A
git diff --cached --quiet || git commit -m "pre: tests-to-spec checkpoint"
git tag tests-to-spec-pre-$(date +%Y%m%d-%H%M%S)
```

### Step 1: 対象選定

**ディレクトリ指定の場合:**
- 指定ディレクトリ内の `*.rs`（`#[cfg(test)]` インライン or 統合テスト）を収集
- 除外: `_archive/`, `target/`, `.refactor-tests/`, `.tests-from-spec/`

**--from-refactor-testsの場合:**
- `tests/.refactor-tests/summary.md` を読み込み
- S1（spec未参照）判定のファイルを対象リストに追加
- should/ ディレクトリ内の対応する分析ファイルも読み込み、追加コンテキストとする

### Step 2: 分析（1ファイル = 1サブエージェント、並列実行）

各テストファイルについて問題の特定・原因調査・対処方針の立案を行う。

1. **テストファイルを読み込み**
   - `mod tests` / `#[test] fn` 構造の把握
   - `use` 文からテスト対象ソースを特定
   - `Spec:` ヘッダーの有無
   - テストケース数

2. **ソースコードを読み込み**（補助情報）
   - 公開 IF の型情報（`pub fn`, `pub struct`, trait）
   - Runtime 判定（Rust / TypeScript）

3. **対応 spec を検索**
   - `documents/spec/` 内で Grep（ソースファイル名、モジュール名）
   - 見つかった場合は読み込み

4. **A〜E 分類**

5. **Case に応じた詳細分析**
   - Case A: テストから抽出可能な仕様の概要
   - Case B: 具体的な乖離箇所（テスト側 vs spec 側）
   - Case C: spec に不足しているセクション・ケース
   - Case D: 推定されるマッピング先
   - Case E: 構造差異の詳細

6. **対処方針の立案**
   - 具体的な修正内容を記述

7. **分類不能な場合**
   - A〜E に当てはまらない問題は「要相談」として記録

**結果出力:** `documents/.spec-from-tests/analysis/` に書き出す。

### Step 3: spec生成/修正（1ファイル = 1サブエージェント、並列実行）

分析結果に基づき、Case に応じた spec を生成する。

#### Case A: 新規 spec 生成

1. テストファイルを一次情報源として spec-v2.1 を生成
2. ソースコードを読み、Contract の型情報と Meta を補完
3. `documents/.spec-from-tests/generated/{カテゴリ}/{ファイル名}.md` に出力

#### Case B: spec 修正

1. 既存 spec とテストの乖離箇所を特定
2. テストを正として修正版 spec を生成
3. `documents/.spec-from-tests/generated/{カテゴリ}/{ファイル名}.md` に出力

#### Case C: spec 追記

1. 既存 spec に不足しているセクション・ケースを生成
2. `documents/.spec-from-tests/generated/{カテゴリ}/{ファイル名}-supplement.md` に出力
3. フロントマターに `Type: Supplement` と `Target: {既存specパス}` を記載

#### Case D: マッピング修正

- spec の生成は不要（既存 spec が存在する）
- 適用時にテストの `Spec:` ヘッダー追加 + 必要なら spec の Source 追記

#### Case E: 構造再構成

1. 既存 spec の構造をテスト構造に合わせて再構成
2. `documents/.spec-from-tests/generated/{カテゴリ}/{ファイル名}.md` に出力

### Step 4: サマリー生成

`documents/.spec-from-tests/summary.md` を生成:
- 統計（Case A〜E の件数、clean 件数）
- 各テストの分類結果と対処内容
- 「要相談」項目（あれば）

### Step 5: ユーザーへの概要提示と許可取得

**重要: 適用前に必ずユーザーの許可を得る。**

ユーザーに以下を提示:
1. 統計サマリー（Case 分布）
2. 各テストの分類と対処内容の概要
3. 「要相談」項目の相談
4. 許可を得てから適用へ

### Step 6: 適用（許可後）

1. **Case A**: `.spec-from-tests/generated/` から `documents/spec/` にコピー
2. **Case B**: 既存 spec を修正版で置換
3. **Case C**: 既存 spec にマージ（末尾に追記 or セクション統合）
4. **Case D**: テストに `Spec:` ヘッダー追加 + spec の Source 追記
5. **Case E**: 既存 spec を再構成版で置換
6. 既存 spec の `更新日:` を更新

### Step 7: クリーンアップ（許可後）

適用完了後、ユーザーの許可を得てから:

1. `documents/.spec-from-tests/` ディレクトリを削除

### Step 8: Post - git commit & tag（完了記録）

```bash
git add -A
git commit -m "tests-to-spec: {N} specs updated ({A}A {B}B {C}C {D}D {E}E)"
git tag tests-to-spec-post-$(date +%Y%m%d-%H%M%S)
```

## 出力構造

```
documents/
├── .spec-from-tests/
│   ├── summary.md              # 全体サマリー
│   ├── analysis/               # 分析結果（Case分類 + 詳細）
│   │   ├── utils/
│   │   │   ├── editor-badge-overlay.md
│   │   │   └── sync-store.md
│   │   └── journal/
│   │       └── task-count.md
│   └── generated/              # 生成/修正された spec（確認待ち）
│       ├── utils/
│       │   ├── editor-badge-overlay.md      # Case A: 新規
│       │   └── sync-store.md               # Case A: 新規
│       └── journal/
│           └── task-count-supplement.md     # Case C: 追記
└── spec/                       # 適用先
```

## summary.md フォーマット

```markdown
# Tests to Spec Summary

実行日: {YYYY-MM-DD}

## Spec一覧

| Test | Source | Spec | Case | Action |
|------|--------|------|------|--------|
| {テストファイルパス} | {ソースコードパス} | {specファイルパス} | {A/B/C/D/E/clean} | {-/generated/updated/mapped} |
```

| 列 | 説明 |
|----|------|
| Test | tests/ からの相対パス |
| Source | code/ からの相対パス |
| Spec | documents/spec/ からの相対パス |
| Case | A〜E の分類、問題なしは clean |
| Action | -(何もしない), generated(新規), updated(修正/追記), mapped(マッピング追加) |

## サブエージェントへの指示

### 分析フェーズ

```
テストファイル分析タスク

## 入力
- テストファイル: {絶対パス}
- specディレクトリ: {documents/spec/の絶対パス}
- codeディレクトリ: {code/の絶対パス}

## 手順
1. テストファイル（`*.rs`、`#[cfg(test)]` インライン or 統合テスト）をReadで読み込む
2. `use` 文からテスト対象のソースファイルを特定し、Readで読み込む
3. documents/spec/ 内で関連specをGrepで検索、見つかればReadで読む
4. 以下の基準で Case A〜E に分類:
   - A: 対応specが見つからない
   - B: specが見つかったが、テストの検証内容と食い違いがある
   - C: specが見つかったが、テストが検証する範囲をカバーしきれていない
   - D: specは存在しそうだがテストの Spec: ヘッダーがなく対応が不明確
   - E: specのセクション構造とテストの構造が対応していない
   - clean: 問題なし
5. A〜Eに分類できない問題は「要相談」として記録
6. Case に応じた詳細分析と対処方針を記述

## 出力
Writeで以下に出力:
{documents_dir}/.spec-from-tests/analysis/{テスト相対パス}.md

フォーマット:
---
Test: {テスト相対パス}
Source: {ソース相対パス}
Spec: {spec相対パス or "なし"}
Case: {A/B/C/D/E/clean}
---

# {テストファイル名}

## 分類
Case {X}: {問題名}

## 詳細分析
{問題の詳細、該当箇所、根拠}

## 対処方針
{具体的な修正内容}

## 戻り値
`{Case}:{テスト相対パス}` または `clean:{テスト相対パス}`
```

### 生成フェーズ（Case A: 新規spec）

```
spec生成タスク（テストから）

## 入力
- テストファイル: {絶対パス}
- ソースコード: {絶対パス}
- 分析結果: {分析ファイルの絶対パス}

## 手順
1. テストファイルをReadで読み込む
2. ソースコードをReadで読み込む（Contract補完用）
3. 分析結果をReadで読み込む
4. spec-v2.1 形式で spec を生成:

   ### 0. Meta
   - Source: `use` パスから特定
   - Runtime: Rust / TypeScript
   - Test Type: Unit / Integration

   ### 1. Contract
   - テストの `use` 対象モジュール、trait の型 → interface 定義
   - ソースコードの `pub fn`, `pub struct`, trait で補完
   - テストが検証している公開 IF のみ記載

   ### 2. State
   - `mod tests` 内のテスト関数のグルーピングから状態遷移を推定
   - 状態遷移がないモジュール（純粋関数等）は省略可

   ### 3. Logic
   - 各 `#[test] fn` を Decision Table の1行に変換
   - テスト関数名 → Case ID + 説明
   - テストの入力値 → Input
   - `assert_eq!` / `assert!` の期待値 → Expected

   ### 4. Side Effects
   - モック対象、外部依存 → 副作用一覧

5. フロントマターを含める:
   ---
   作成日: {今日}
   更新日: {今日}
   確認日: -
   使用終了日: -
   Format: spec-v2.1
   Source: {ソースコードの相対パス}
   ---

## 出力
Writeで以下に出力:
{documents_dir}/.spec-from-tests/generated/{カテゴリ}/{spec名}.md

## 戻り値
`generated:{spec相対パス}`
```

### 生成フェーズ（Case C: 追記用spec）

```
spec追記生成タスク（テストから）

## 入力
- テストファイル: {絶対パス}
- 既存spec: {既存specの絶対パス}
- 分析結果: {分析ファイルの絶対パス}

## 手順
1. テストファイルをReadで読み込む
2. 既存specをReadで読み込み、形式を確認
3. 分析結果をReadで読み込む
4. テストにあるがspecに記載のない仕様のみを抽出・生成
5. フロントマターを含める:
   ---
   作成日: {今日}
   更新日: {今日}
   確認日: -
   使用終了日: -
   Type: Supplement
   Target: {既存specの相対パス}
   Source: {ソースコードの相対パス}
   ---

## 出力
Writeで以下に出力:
{documents_dir}/.spec-from-tests/generated/{カテゴリ}/{ファイル名}-supplement.md

## 戻り値
`generated:{spec相対パス}`
```

## 除外対象

- `_archive/` - アーカイブ済みテスト
- `target/` - Rustビルド成果物
- `.refactor-tests/` - refactor-tests 作業ディレクトリ
- `.tests-from-spec/` - spec-to-tests 作業ディレクトリ

## 注意事項

- **テストが一次情報源**: spec の内容はテストファイルから導出する。ソースコードは型情報と Meta 情報の補完にのみ使用
- **テストが検証していないことは書かない**: ソースコードにあってもテストが検証していない機能は spec に含めない
- **分類不能はユーザーに相談**: A〜E に当てはまらない問題は「要相談」として記録し、ユーザーに報告する。「その他」に押し込めない
- **確認フロー必須**: 生成 spec は `.spec-from-tests/generated/` に出力し、ユーザーに概要を提示して許可を得てから適用
- **形式統一**: 生成する spec は spec-v2.1 形式で統一
- **出所明記**: 生成 spec のフロントマターに `Source:` でソースコードファイルを記録
- **追記用の区別**: 追記用 spec は `-supplement.md` サフィックス + `Type: Supplement` で区別
- **Spec ヘッダー追加**: 適用後、対応するテストファイルに `// Spec: documents/spec/{path}` ヘッダーを追加
