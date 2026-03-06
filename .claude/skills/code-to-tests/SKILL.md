---
name: code-to-tests
description: コードを起点にテストカバレッジを検査し、不足分を生成・適用・実行するスキル。
---

# Code to Tests

コードを起点にテストカバレッジを検査し、不足分を生成・適用・実行するスキル。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: コードファイルを収集
3. **分析**: サブエージェント並列で公開IF抽出・テスト対応判定（1 code = 1 agent）
4. **生成+適用**: サブエージェント並列でテスト生成 → `code/tauri/src/` にインラインテストとして直接配置（1 code = 1 agent）
5. **テスト実行**: `cd code/tauri && cargo test --lib`（ユニットテスト）
6. **サマリー**: 統計・一覧・実行結果を `tests/.tests-from-code/summary.md` に出力
7. **ユーザー確認**: 概要提示 → 判断（commit / 修正 / revert）
8. **クリーンアップ**: 作業ディレクトリ削除
9. **Post**: git commit & tag（完了記録）

## 概要

| 項目 | 内容 |
|------|------|
| 入力 | コードディレクトリ/ファイル or `--from-refactor-code` or `--from-tests-to-spec` |
| 出力 | `tests/.tests-from-code/` （分析）+ `code/tauri/src/` （インラインテスト直接配置） |
| 一次情報源 | コードファイル |
| 補助情報源 | 既存テスト（パターン・構造の参照） |
| テストランナー | `cargo test` |

## 位置づけ

3側面一致原則（code ↔ test ↔ spec）の6辺のうち、code → tests の直接辺。

| スキル | 起点 | 終点 | 一次情報源 | 問い |
|--------|------|------|-----------|------|
| spec-to-tests | spec | tests | spec | 「spec の仕様にテストがあるか？」 |
| tests-to-code | tests | code | tests | 「テストが検証する内容とコードが一致するか？」 |
| **code-to-tests** | **code** | **tests** | **code** | **「コードの公開 IF にテストがあるか？」** |

### spec-to-tests との違い

- `spec-to-tests`: spec が正。spec に書いてあるのにテストがない → 生成
- `code-to-tests`: code が正。code に pub があるのにテストがない → 検出/生成

spec が存在しない・古い・不完全な場合でも、code から直接テストカバレッジを検証できる。

### spec を使わない

spec は補助情報源として使わない。code のみから分析・生成する。
spec を介在させたい場合は `code-to-spec` → `spec-to-tests` のチェーンを使う。

## 入力オプション

### オプションA: ディレクトリ/ファイル指定

```
code-to-tests code/tauri/src
code-to-tests code/tauri/src/timer.rs
```

指定パス配下のコードファイル（`*.rs`、`*.svelte`、`*.ts`）を対象とする。

### オプションB: refactor-codeの結果を使用

```
code-to-tests --from-refactor-code
```

`code/.refactor/` の結果から対象コードファイルを取得。

### オプションC: tests-to-specの結果を使用

```
code-to-tests --from-tests-to-spec
```

`_documents/.spec-from-tests/summary.md` の Source 列からコードファイルを取得。

## 分類

### ファイルレベル分類

| 分類 | 条件 | アクション |
|------|------|-----------|
| **covered** | `#[cfg(test)]` モジュールが存在し、公開 IF を網羅 | スキップ（サマリーに記録） |
| **partial** | `#[cfg(test)]` モジュールが存在するが、未カバーの公開 IF がある | supplement テスト生成 |
| **missing** | `#[cfg(test)]` モジュールが存在しない | フルテスト生成 |

### メソッドレベル分類（partial 内の詳細）

各公開 IF に対して:

| Level | 名前 | 条件 | 生成優先度 |
|-------|------|------|-----------|
| **L1** | untested | テスト自体がない（関数名がテストに登場しない） | 高 |
| **L2** | undertested | テストはあるが主要分岐が未カバー | 中 |
| - | tested | 公開 IF がテストされており、主要分岐もカバー | - |

L1 は L2 より深刻。L1 のメソッドが残っている限りファイルは partial。
全メソッドが tested になればファイルは covered。

**L2 判定の基準**（コードの分岐構造から判定）:

- 早期 return（ガード節）がテストされていない
- if/else の片方のみテスト
- エラーパス（`return Err(...)`, `eprintln!`, `panic!`）が未テスト
- ループの 0回/1回/N回 のうち一部のみ

### 「公開 IF」の定義

code-to-tests が検査する対象:

1. **`pub` な関数/構造体/定数**
2. **`pub` メソッド**（`impl` ブロック内の `pub fn`）
3. **Tauri コマンド**（`#[tauri::command]` 属性付き関数）
4. **イベント emit/listen**（`app.emit()`, `app.listen()` のイベント名）

検査**しない**もの:
- `pub(crate)` や非公開関数（間接テスト対象）
- 型定義のみのファイル
- フロントエンド（Svelte/TS）は `svelte-check` による型チェックのみ

## ワークフロー

### Step 0: Pre - git commit & tag（ロールバックポイント）

**スキル開始前に現在の状態を保存する。**

```bash
git add -A
git diff --cached --quiet || git commit -m "pre: code-to-tests checkpoint"
git tag code-to-tests-pre-$(date +%Y%m%d-%H%M%S)
```

### Step 1: 対象選定

**ディレクトリ/ファイル指定の場合:**
- 指定パス内の `*.rs` を収集（Rust ユニットテスト対象）
- 除外: `target/`, `tests/integration/`（統合テストは別管理）、データファイル

**--from-refactor-codeの場合:**
- `code/.refactor/` の結果から対象ファイルリストを取得

**--from-tests-to-specの場合:**
- `_documents/.spec-from-tests/summary.md` を読み込み
- Source 列からコードファイルを取得

### Step 2: 分析（1ファイル = 1サブエージェント、並列実行）

各コードファイルについて:

1. **コードファイルを読み込み**
   - `pub fn`, `pub struct`, `pub const` を抽出
   - `impl` ブロック内の `pub fn` メソッドを抽出
   - `#[tauri::command]` 関数を抽出
   - 各公開 IF の分岐構造を分析

2. **対応テストを検索**
   - 同一ファイル内の `#[cfg(test)] mod tests` ブロックを確認
   - テスト関数内の `use super::*` で対象モジュールを確認
   - 統合テスト `tests/integration/` 内での参照も確認

3. **ファイルレベル判定**
   - `#[cfg(test)]` モジュールが見つからない → **missing**
   - `#[cfg(test)]` モジュールが見つかった → 各公開 IF のメソッドレベル判定へ

4. **メソッドレベル判定**（テストモジュールが存在する場合）
   - 各公開 IF について:
     - 関数名がテストに登場しない → **L1 (untested)**
     - テストはあるが主要分岐が未カバー → **L2 (undertested)**
     - テスト済み → **tested**
   - L1 or L2 が1つでもあれば → ファイルは **partial**
   - 全て tested → ファイルは **covered**

5. **結果出力**
   - `tests/.tests-from-code/analysis/{ファイル名}.md` に書き出す

### Step 3: 生成 + 適用（partial/missing のみ、1ファイル = 1サブエージェント、並列実行）

covered はスキップ。partial/missing のみ処理する。

#### missing の場合（フルテスト生成）

1. コードの `pub` 関数・メソッドを全て抽出
2. 各公開関数の分岐構造を分析
3. 既存テストモジュール（例: `timer.rs` 内の `mod tests`）からプロジェクトのテストパターンを参照
4. 対象ファイル末尾に `#[cfg(test)] mod tests { ... }` を**直接追加**

#### partial の場合（supplement テスト生成）

1. L1/L2 の関数のみ対象
2. 既存テストの構造（ヘルパー関数、セットアップパターン）を踏襲
3. 既存の `#[cfg(test)] mod tests` ブロック内にテスト関数を**追加**
4. 追加テストにコメント: `// Supplement: L1/L2 gaps from code-to-tests`

### Step 4: テスト実行

生成テスト含む全テストを実行:

```bash
cd code/tauri && cargo test --lib
```

結果を記録（pass/fail ファイル、失敗内容）。

### Step 5: サマリー生成

`tests/.tests-from-code/summary.md` を生成（実行結果込み）。

### Step 6: ユーザー確認

**重要: ユーザーに報告し判断を仰ぐ。**

ユーザーに以下を提示:
1. 統計サマリー（covered/partial/missing、L1/L2 件数）
2. テスト実行結果（pass/fail）
3. 各ファイルの分類と対処内容の概要

判断:
- **全 pass → commit**
- **fail あり → 対処判断**:
  - テスト修正: 生成テストのバグ → 修正して再実行
  - コードバグ発見: テストが正しくコードにバグがある → コード修正は別タスク
  - revert: テストを削除（git tag からロールバック可能）

### Step 7: クリーンアップ（許可後）

適用完了後、ユーザーの許可を得てから:

1. `tests/.tests-from-code/` ディレクトリを削除

### Step 8: Post - git commit & tag（完了記録）

```bash
git add -A
git commit -m "code-to-tests: {N} tests generated ({covered}c {partial}p {missing}m, {L1}L1 {L2}L2)"
git tag code-to-tests-post-$(date +%Y%m%d-%H%M%S)
```

## 出力構造

```
tests/
└── .tests-from-code/
    ├── summary.md              # 全体サマリー（実行結果込み）
    └── analysis/               # カバレッジ分析結果
        ├── timer.md
        └── presence.md

code/tauri/src/
├── timer.rs                    # 既存 #[cfg(test)] に supplement 追加
├── presence.rs                 # missing → #[cfg(test)] mod tests 新規追加
└── lib.rs                      # インラインテスト
```

## summary.md フォーマット

```markdown
# Code to Tests Summary

実行日: {YYYY-MM-DD}

## 統計

| Status | Count |
|--------|-------|
| covered | N |
| partial | N |
| missing | N |

| Level | Count |
|-------|-------|
| L1 (untested) | N functions |
| L2 (undertested) | N functions |

## テスト実行結果

| 結果 | Count |
|------|-------|
| pass | N files |
| fail | N files |

## 一覧

| Code | Test | Status | L1 | L2 | 生成 | cargo test |
|------|------|--------|----|----|------|------------|
| tauri/src/timer.rs | inline #[cfg(test)] | partial | try_transition | advance(境界) | supplement | pass |
| tauri/src/presence.rs | - | missing | - | - | new | pass |
| tauri/src/lib.rs | inline #[cfg(test)] | covered | - | - | - | - |
```

## 分析ファイルフォーマット

```markdown
---
Code: {コード相対パス}
Test: {インラインテスト有無 or "なし"}
Status: {covered/partial/missing}
L1: {untested 関数数}
L2: {undertested 関数数}
---

# {コードファイル名}

## 公開 IF

| # | 名前 | 種別 | Level | 備考 |
|---|------|------|-------|------|
| 1 | advance | pub fn | tested | - |
| 2 | try_transition | pub fn | L2 | Idle→Work 遷移パスが未テスト |
| 3 | apply_settings | pub fn | L1 | テストなし |

## L1 詳細（untested）

### apply_settings
- 行: L130-145
- シグネチャ: `pub fn apply_settings(&mut self, settings: TimerSettings)`
- 分岐: phase 判定(L131)、duration 更新(L136)、elapsed リセット(L140)
- 推奨テストケース:
  1. Work フェーズ中に duration 変更
  2. Break フェーズ中に duration 変更
  3. Idle 状態での適用

## L2 詳細（undertested）

### try_transition
- テスト箇所: timer.rs #[cfg(test)] 内 test_try_transition_*
- テスト済み分岐: Work→Break, Break→Idle
- 未テスト分岐:
  1. Idle→Work 遷移条件
  2. elapsed == duration の境界値
```

## テスト生成ルール

### テストランナー・アサーション

| 項目 | 方針 |
|------|------|
| 一次情報源 | コードファイル |
| 補助情報源 | 既存テスト（パターン・構造の参照） |
| テストランナー | `cargo test` |
| アサーション | `assert!`, `assert_eq!`, `assert_ne!` |
| インポート | `use super::*` or `use crate::` |
| モック | trait + テスト用実装、または mockall クレート |
| 命名 | 既存テストの命名パターンに従う |

### 期待値導出の原則

コードが一次情報源である以上、テストの期待値はコードのロジックから**厳密に導出**しなければならない。推測・仮定は禁止。

**ロジックトレース義務**:
- 分岐・ループを含む関数のテストでは、具体的な入力値でコードを1行ずつステップ実行し、期待値を導出する
- 「こうなるはず」という推測ではなく、コードの各行を辿った結果をコメントに記録する
- テストコードに**導出過程をコメントで明記**する（レビュー可能にするため）

```rust
// BAD: 推測で期待値を決定
assert_eq!(timer.elapsed, 0);

// GOOD: コードをトレースして期待値を導出し、過程をコメントに残す
// advance() L45: elapsed += 1 → 1
// try_transition() L52: elapsed(1) < duration(25) → no transition
// → elapsed stays 1
assert_eq!(timer.elapsed, 1);
```

**イベント emit のテスト**:
- emit されるイベント名とペイロードをコードから確認する
- ハンドラ内部で呼ばれる関数の副作用まで追跡する

### テスト間の状態管理

各 `#[test]` 関数は独立したセットアップを持つ。Rust のテストはデフォルトで並列実行されるため、共有状態に注意する。

**ルール**:
- 各テスト関数内でテスト対象の構造体を新規作成する
- グローバル状態（static mut 等）を使うテストでは `#[serial]` を検討する

```rust
// BAD: テスト間で状態を共有
static mut SHARED: Option<Timer> = None;

// GOOD: 各テストで独立にセットアップ
#[test]
fn test_advance() {
    let mut timer = Timer::new(default_settings());
    timer.advance();
    assert_eq!(timer.elapsed, 1);
}
```

### L1 生成（untested 関数）

公開 IF の正常系 + 全ガード節 + 主要エラーパスをテスト:

```rust
// テスト生成例: L1 (untested) 関数
#[test]
fn test_apply_settings_during_work() {
    // 正常系: Work フェーズ中に設定変更
    let mut timer = Timer::new(default_settings());
    timer.start();
    let new_settings = TimerSettings { work_duration: 30, ..default_settings() };
    timer.apply_settings(new_settings);
    assert_eq!(timer.work_duration, 30);
}

#[test]
fn test_apply_settings_idle() {
    // ガード節: Idle 状態での適用
    let mut timer = Timer::new(default_settings());
    timer.apply_settings(TimerSettings { work_duration: 30, ..default_settings() });
    assert_eq!(timer.work_duration, 30);
}
```

### L2 生成（undertested 関数）

分析で特定された未テスト分岐のみをテスト:

```rust
// テスト生成例: L2 (undertested) 関数
// Supplement: L1/L2 gaps from code-to-tests

#[test]
fn test_try_transition_idle_to_work() {
    // 未テスト分岐: Idle→Work 遷移
    let mut timer = Timer::new(default_settings());
    timer.start();
    assert_eq!(timer.phase, Phase::Work);
}

#[test]
fn test_try_transition_boundary() {
    // 未テスト分岐: elapsed == duration の境界値
    let mut timer = Timer::new(TimerSettings { work_duration: 1, ..default_settings() });
    timer.start();
    timer.advance();
    assert_eq!(timer.phase, Phase::Break);
}
```

### テストのモッキングパターン

| 対象 | 手法 | 理由 |
|------|------|------|
| trait 依存 | trait + テスト用 struct 実装 | Rust の標準的パターン |
| Tauri AppHandle | mockall の `#[automock]` or テスト用 wrapper | Tauri API は trait 化して差し替え |
| 外部 API 呼び出し | trait 抽象化 + テスト用実装 | ネットワーク依存を排除 |
| ファイル I/O | `tempfile` クレート + trait 抽象化 | テスト環境の独立性 |

**禁止**: 手書き部分モック。trait ベースの差し替えを使う。

### テスト配置

| ケース | 配置方法 |
|--------|----------|
| missing → 新規 | 対象ファイル末尾に `#[cfg(test)] mod tests { ... }` を追加 |
| partial → supplement | 既存の `#[cfg(test)] mod tests` ブロック内にテスト関数を追加 |

### テストヘッダー

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Tests for: {ソースファイル名}
    // Generated by: code-to-tests
    // Created: {YYYY-MM-DD}

    // ... テスト関数 ...
}
```

## テスト対応関係の検出

コードファイルから対応するテストを以下の順序で探索:

1. **インラインテスト確認**（最も確実）: 同一ファイル内の `#[cfg(test)] mod tests` ブロック
2. **統合テスト確認**: `tests/integration/` 内で `use` 文による参照
3. **関数名検索**: テストコード内で対象関数名が呼び出されているか

## fail 時の対処

生成テストが fail した場合:

1. サマリーに fail ファイルと失敗内容を記録
2. ユーザーに報告し判断を仰ぐ:
   - **テスト修正**: 生成テストのバグ → 修正して再実行
   - **コードバグ発見**: テストが正しくコードにバグがある → コード修正は別タスク
   - **revert**: テストを削除（git tag からロールバック可能）

## サブエージェントへの指示

### 分析フェーズ

```
コードファイル分析タスク

## 入力
- コードファイル: {絶対パス}
- テスト確認先: 同一ファイル内の #[cfg(test)] + tests/integration/

## 手順
1. コードファイルをReadで読み込む
2. 公開IFを抽出:
   - pub fn, pub struct, pub const
   - impl ブロック内の pub fn メソッド
   - #[tauri::command] 関数
   - イベント emit/listen のイベント名
3. 各公開IFの分岐構造を分析:
   - ガード節（早期return）
   - if/else, match 分岐
   - エラーパス（return Err(...), eprintln!, panic!）
   - ループ
4. 対応テストを検索:
   a. 同一ファイル内の #[cfg(test)] mod tests ブロックを確認
   b. tests/integration/ 内で use 文による参照を確認
   c. 見つかったテストをReadで読み込む
5. ファイルレベル判定:
   - #[cfg(test)] なし → missing
   - #[cfg(test)] あり → 各関数のLevel判定へ
6. メソッドレベル判定（テストモジュールがある場合）:
   - 関数名がテストに登場しない → L1 (untested)
   - テストはあるが主要分岐が未カバー → L2 (undertested)
   - 十分にテスト済み → tested
7. ファイル判定:
   - L1 or L2 が1つでもあれば → partial
   - 全て tested → covered

## 出力
Writeで以下に出力:
tests/.tests-from-code/analysis/{ファイル名}.md

フォーマット:
---
Code: {コード相対パス}
Test: {インラインテスト有無 or "なし"}
Status: {covered/partial/missing}
L1: {untested 関数数}
L2: {undertested 関数数}
---

# {コードファイル名}

## 公開 IF

| # | 名前 | 種別 | Level | 備考 |
|---|------|------|-------|------|
| ... |

## L1 詳細（untested）
（各L1関数の行範囲、シグネチャ、分岐、推奨テストケース）

## L2 詳細（undertested）
（各L2関数のテスト箇所、テスト済み分岐、未テスト分岐）

## 戻り値
`{Status}:{コード相対パス}:{L1数}:{L2数}` または `covered:{コード相対パス}`
```

### 生成フェーズ（missing: フルテスト）

```
テスト生成タスク（missing）

## 入力
- コードファイル: {絶対パス}
- 分析結果: {分析ファイルの絶対パス}
- 参考テスト: {同プロジェクトの既存テスト付きファイル絶対パス}（パターン参照用）

## 手順
1. コードファイルをReadで読み込む
2. 分析結果をReadで読み込む
3. 参考テストをReadで読み込む（スタイル・パターンの参照）
4. 全公開関数のテストを生成:
   - 正常系
   - 全ガード節
   - 主要エラーパス
5. コードファイル末尾に #[cfg(test)] mod tests ブロックを追加

## テストモジュールの要件
- ヘッダー: コメントで Generated by: code-to-tests, Created: 日付
- use super::* でモジュール内の全アイテムをインポート
- 必要に応じて use crate:: で他モジュールの型をインポート
- モック: trait + テスト用実装を使用、手書き部分モック禁止
- 命名: 既存テストの命名パターンに従う（test_ プレフィックス）

## 期待値導出（必須）
テストの各 assert!() / assert_eq!() に対して、コードのロジックをトレースして期待値を導出すること。
- 分岐・ループは具体値で展開し、結果をコメントに記録する
- 推測で期待値を決めてはならない。コードの行番号を辿って導出する
- イベント emit はペイロードの内容まで追跡する

## テスト間の独立性（必須）
各 #[test] 関数は独立したセットアップを持つこと。
- テスト対象の構造体は各テスト関数内で新規作成する
- グローバル状態を共有しない

## 出力
Editで対象ファイル末尾に #[cfg(test)] mod tests { ... } を追加

## 戻り値
`generated:{コード相対パス}:{テストケース数}`
```

### 生成フェーズ（partial: supplement テスト）

```
テスト生成タスク（partial supplement）

## 入力
- コードファイル: {絶対パス}
- 分析結果: {分析ファイルの絶対パス}

## 手順
1. コードファイルをReadで読み込む（既存テストモジュール含む）
2. 分析結果をReadで読み込む
3. L1/L2 の関数のみ対象にテストを生成:
   - L1: 正常系 + 全ガード節 + 主要エラーパス
   - L2: 分析で特定された未テスト分岐のみ
4. 既存テストの構造（ヘルパー関数、セットアップパターン）を踏襲
5. 既存の #[cfg(test)] mod tests ブロック内にテスト関数を追加
6. 追加テストにコメント: // Supplement: L1/L2 gaps from code-to-tests

## 期待値導出（必須）
テストの各 assert!() / assert_eq!() に対して、コードのロジックをトレースして期待値を導出すること。
- 分岐・ループは具体値で展開し、結果をコメントに記録する
- 推測で期待値を決めてはならない。コードの行番号を辿って導出する
- イベント emit はペイロードの内容まで追跡する

## テスト間の独立性（必須）
各 #[test] 関数は独立したセットアップを持つこと。
- テスト対象の構造体は各テスト関数内で新規作成する
- グローバル状態を共有しない

## 出力
Editで既存の mod tests ブロック内にテスト関数を追加

## 戻り値
`generated:{コード相対パス}:{テストケース数}`
```

## 除外対象

- `target/` - ビルド出力
- `tests/integration/` - 統合テストは別管理
- `node_modules/`
- フロントエンド（`*.svelte`, `*.ts`）は `svelte-check` のみ（ユニットテスト対象外）
- データファイル（`.json`, `.csv` 等）

## 注意事項

- **コードが一次情報源**: テストはコードファイルから導出する。spec は参照しない
- **spec を使わない**: spec を介在させたい場合は `code-to-spec` → `spec-to-tests` のチェーンを使う
- **インライン配置**: 生成テストは対象 `.rs` ファイルの `#[cfg(test)] mod tests` にインラインで配置する
- **テスト実行必須**: 配置後に `cargo test --lib` を実行し、結果をサマリーに記録する
- **ロールバック**: Pre の git tag でロールバック可能
- **既存スタイル踏襲**: 生成するテストは既存テスト（例: `timer.rs`）のスタイル・パターンに合わせる
- **trait ベースモック必須**: モックは trait + テスト用実装を使用。手書き部分モック禁止
- **`use super::*`**: テストモジュール内では `use super::*` でインポート
- **L1 優先**: L1（untested）は L2（undertested）より深刻。L1 を優先的に対処する
- **supplement の区別**: supplement テストはコメントで区別（`// Supplement: L1/L2 gaps from code-to-tests`）
- **クリーンアップは許可後**: `.tests-from-code/` の削除はユーザーの許可を得てから実行する
- **期待値は推測禁止**: テストの assert の値はコードをトレースして導出する。分岐・ループは具体値で展開し、導出過程をコメントに残す
- **テスト間の独立性**: 各 `#[test]` 関数は独立したセットアップを持つ。グローバル状態を共有しない
