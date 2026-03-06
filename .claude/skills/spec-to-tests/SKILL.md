---
name: spec-to-tests
description: 仕様からテストを生成する。「spec-to-tests <path>」または「spec-to-tests --from-spec-from-code」で使用。対応するtestsの有無を調査し、漏れがあれば生成。partial判定のものも不足分を生成する。
---

# Spec to Tests

仕様からテストを生成するスキル（#4）。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: specファイルを収集
3. **調査**: サブエージェント並列でspec↔test対応を調査（1 spec = 1 agent）
4. **生成**: サブエージェント並列でテストを生成（1 spec = 1 agent）
5. **サマリー**: 統計・一覧を `.tests-from-spec/summary.md` に出力
6. **ユーザー確認**: 概要提示 → 許可取得
7. **適用**: 許可後に `code/tauri/src/`（Rustインライン）or `tests/integration/` へ配置
8. **クリーンアップ**: 作業ディレクトリ削除（summaryは残す）
9. **Post**: git commit & tag（完了記録）

## 概要

| 項目 | 内容 |
|------|------|
| 入力 | specディレクトリ or `--from-spec-from-code` |
| 出力 | `tests/.tests-from-spec/` → 確認後 `code/tauri/src/`（Rustインライン）or `tests/integration/` |
| 形式 | Runtime に応じて Rust `#[cfg(test)]` インライン or TypeScript（Svelte/TS用） |

## 入力オプション

### オプションA: ディレクトリ指定

```
spec-to-tests documents/spec/panel
spec-to-tests documents/spec/auth
```

指定ディレクトリ内のspecファイルを対象とする。

### オプションB: code-to-specの結果を使用

```
spec-to-tests --from-spec-from-code
```

`documents/.spec-from-code/summary.md` から生成されたspecリストを取得。
code-to-specで生成したspecを優先的に処理できる。

## ワークフロー

### Step 0: Pre - git commit & tag（ロールバックポイント）

**スキル開始前に現在の状態を保存する。**

```bash
# 未コミットの変更があればコミット
git add -A
git diff --cached --quiet || git commit -m "pre: spec-to-tests checkpoint"

# タグを作成（ロールバック用）
git tag spec-to-tests-pre-$(date +%Y%m%d-%H%M%S)
```

問題発生時に `git reset --hard spec-to-tests-pre-XXXXXXXX` でロールバック可能にする。

### Step 1: 対象選定

**ディレクトリ指定の場合:**
```bash
# 指定ディレクトリ内のspecファイルを収集
# 対象: *.md ファイル（Format: が記載されているもの）
```

**--from-spec-from-codeの場合:**
```bash
# documents/.spec-from-code/summary.md を読み込み
# 全specファイルを対象リストに追加
```

**テスト生成対象の判定:**

| Check | 処理 | 理由 |
|-------|------|------|
| missing | 新規test生成 | 対応するテストが存在しない |
| partial | 追記用test生成 | テストは存在するが一部シナリオが未カバー |
| covered | **スキップ** | テストが存在し主要シナリオがカバー済み（生成不要） |

**重要**: coveredのspecはテスト完備のため、Step 2以降の処理対象から除外する。ただし、summaryには全specの状況を記録する。

### Step 2: tests調査（1ファイル = 1サブエージェント、並列実行）

各specファイルについて:

1. **specを読み込み**
   - Format（Gherkin/StateMachine/Contract/DecisionTable）を特定
   - テスト対象（Source:）を確認
   - 主要なシナリオ/状態遷移/API を把握

2. **対応testsを検索**
   - `code/tauri/src/` 内のインラインテスト、および `tests/integration/` で関連するテストを検索
   - ファイル名、テスト名でマッチング

3. **カバレッジ判定**
   - **covered**: テストが存在し、主要シナリオがカバーされている
   - **partial**: テストは存在するが、一部シナリオが未テスト（不足内容を明記）
   - **missing**: 対応するテストが存在しない

4. **結果出力**
   - 調査結果を `tests/.tests-from-spec/analysis/` に出力

### Step 3: tests生成（missing + partial両方を処理）

**重要: missingだけでなく、partialも処理する。**

#### missingの場合（新規テスト）

1. specのFormatに応じてテスト構造を決定
2. 完全なテストファイルを生成
3. `tests/.tests-from-spec/generated/{カテゴリ}/{ファイル名}_tests.rs` に出力

#### partialの場合（追記用テスト）

1. 既存テストの構造を確認
2. 不足しているテストケースのみを生成
3. `tests/.tests-from-spec/generated/{カテゴリ}/{ファイル名}_supplement_tests.rs` に出力
4. コメントに `// Supplement for: {既存テストパス}` を記載

### Step 4: サマリー生成

`tests/.tests-from-spec/summary.md` を生成:
- 統計（covered/partial/missing件数、生成したtest件数）
- 新規test一覧
- 追記用test一覧
- 推奨アクション

### Step 5: ユーザーへの概要提示と許可取得

**重要: 適用前に必ずユーザーの許可を得る。**

#### 提示内容

ユーザーに以下の概要を提示する:

1. **統計サマリー**
   - 対象spec数、covered/partial/missing件数
   - 生成したtest件数（新規/追記）

2. **新規testの説明**（各ファイルについて）
   - 対象spec
   - テストケース数
   - 主要なテスト内容

3. **追記用testの説明**（各ファイルについて）
   - 対象spec → ターゲットtest
   - 追加されるテストケースの概要

#### 許可取得

- ユーザーが「更新しましょう」等の許可を出すまで待機
- 質問があれば回答する
- 修正要望があれば対応する

### Step 6: 適用（許可後）

ユーザーの許可を得てから:

1. **新規test（Rust）**: `.tests-from-spec/generated/` の内容を `code/tauri/src/` 内の対応ファイルに `#[cfg(test)] mod tests` として追記
2. **新規test（統合）**: `.tests-from-spec/generated/` から `tests/integration/` にコピー
3. **追記用test**: 既存テストの `mod tests` ブロック内に追記

### Step 7: git commit

適用後、変更をコミット:

```bash
git add code/tauri/src/ tests/integration/ tests/.tests-from-spec/summary.md
git commit -m "spec-to-tests: add {N} tests from spec"
```

### Step 8: クリーンアップ

**コミット後、作業ディレクトリを削除してsummaryのみ残す:**

1. `tests/.tests-from-spec/generated/` を削除
2. `tests/.tests-from-spec/analysis/` を削除
3. `tests/.tests-from-spec/summary.md` は**残す**（何を生成したかの記録）

```bash
rm -rf tests/.tests-from-spec/generated/
rm -rf tests/.tests-from-spec/analysis/
# summary.md は残す
```

**重要**: summaryには生成したファイルの一覧が記録されているため、generated/を削除しても追跡可能。次工程（tests-to-code）が間違った場所を触らないよう、作業ディレクトリは必ず削除する。

### Step 9: Post - git commit & tag（完了記録）

**スキル完了後に結果をコミット & タグ付け。**

```bash
git add -A
git commit -m "spec-to-tests: add {N} tests from spec"
git tag spec-to-tests-post-$(date +%Y%m%d-%H%M%S)
```

## 出力構造

```
tests/
├── .tests-from-spec/
│   ├── summary.md              # 全体サマリー
│   ├── analysis/                # 調査結果
│   │   ├── panel-assemble.md
│   │   └── auth-overlay.md
│   └── generated/               # 生成されたtest（確認待ち）
│       ├── timer/
│       │   └── timer_tests.rs             # 新規test（Rustインライン用）
│       └── lifecycle/
│           └── app_lifecycle_supplement_tests.rs  # 追記用test
└── integration/                 # 統合テスト
    └── app_lifecycle.rs
```

## テスト生成ルール

### Runtime に応じたテスト生成

spec の Meta > Source テーブルから Runtime を読み取り、テスト方式を分岐する。

| Runtime | テストランナー | テスト方式 | 出力先 |
|---------|--------------|------------|--------|
| `Rust` | `cargo test --lib` | `#[cfg(test)] mod tests` インライン | `code/tauri/src/` 内の対応ファイル |
| `JS-ESM` | `npx svelte-check`（型チェックのみ） | TypeScript 型検査 | フロントエンドは型チェックのみ。ロジックテストが必要な場合は Rust 側で検証 |
| 未記載 | — | Source の拡張子から推定 | 推定不可なら報告して保留 |

**1つの spec に複数の Runtime がある場合**: Runtime ごとに別のテストファイルを生成する。

**Runtime 未記載時の推定ルール**:
1. 拡張子が `.rs` → `Rust`
2. 拡張子が `.ts`/`.svelte` → `JS-ESM`（型チェックのみ）
3. 推定不可 → テスト生成を保留して報告

### Rust テストフォーマット（`#[cfg(test)]` インライン）

```rust
// {モジュール名} - {テスト種別}
//
// Tests for: {ソースファイル}
// Spec: {specファイル}
// Runtime: Rust

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_名前() {
        // Arrange
        let state = TimerState::new(/* ... */);

        // Act
        let result = state.advance();

        // Assert
        assert_eq!(result.phase, Phase::Focus);
    }
}
```

### テストのモッキングパターン（Rust）

テスト生成時は以下のパターンに従うこと。

#### trait モック: テスト用の実装を定義

依存関係は trait で抽象化し、テスト用の struct を定義する。

```rust
// プロダクションコード
trait Storage {
    fn save(&self, key: &str, value: &str) -> Result<(), Error>;
    fn load(&self, key: &str) -> Result<String, Error>;
}

// テスト用モック
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockStorage {
        data: RefCell<HashMap<String, String>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self { data: RefCell::new(HashMap::new()) }
        }
    }

    impl Storage for MockStorage {
        fn save(&self, key: &str, value: &str) -> Result<(), Error> {
            self.data.borrow_mut().insert(key.to_string(), value.to_string());
            Ok(())
        }
        fn load(&self, key: &str) -> Result<String, Error> {
            self.data.borrow().get(key).cloned().ok_or(Error::NotFound)
        }
    }
}
```

#### テスト用ヘルパー: ファクトリ関数

テストデータの生成にはヘルパー関数を使う。

```rust
#[cfg(test)]
fn make_timer_state(focus_min: u32, break_min: u32) -> TimerState {
    TimerState::new(focus_min * 60, break_min * 60)
}
```

#### 使い分け表

| 対象 | 手法 | 理由 |
|------|------|------|
| 依存 trait | テスト用 struct + impl | Rust にはリフレクションベースのモックがない |
| 状態生成 | ファクトリ関数 | テストの可読性向上 |
| 時間依存 | 引数で注入 or `advance()` 呼び出し回数で制御 | `std::time` を直接モックしない |
| Tauri コマンド | 統合テスト（`tests/integration/`）で検証 | プロセス起動 + stderr ログアサーション |

### Format別のテスト生成方針

| Format | テスト構造 | テスト内容 |
|--------|-----------|-----------|
| Gherkin | Scenarioごとに `#[test] fn` | Given/When/Then を検証 |
| StateMachine | 状態遷移ごとに `mod` + `#[test]` グループ | 各遷移の前提条件・アクション・結果 |
| Contract | メソッドごとに `mod` + `#[test]` グループ | 入力・出力・エラー条件 |
| DecisionTable | 行ごとに `#[test] fn` | 入力組み合わせ → 期待出力 |

### テストケース命名

```rust
// Rust
// 番号付き: test_{セクション}_{サブセクション}_{ケース}_説明
#[test]
fn test_1_1_1_initial_mode_is_blank() { ... }

// シナリオベース: test_scenario_名前
#[test]
fn test_scenario_open_menu() { ... }

// モジュール内グルーピング
mod phase_transition {
    use super::*;

    #[test]
    fn test_focus_to_break() { ... }

    #[test]
    fn test_break_to_focus() { ... }
}
```

## summary.md フォーマット

テンプレート: `templates/summary.md`

```markdown
# Spec to Tests Summary

実行日: {YYYY-MM-DD}

## Test一覧

| Test | Spec | Source | Check | Action |
|------|------|--------|-------|--------|
| {テストファイルパス} | {specファイルパス} | {ソースコードパス} | {covered/partial/missing} | {-/generated} |
```

**目的**: 次工程（tests-to-code）へのバトン渡し

- **Test**: code/tauri/src/ or tests/integration/ からの相対パス
- **Spec**: documents/spec/ からの相対パス
- **Source**: code/ からの相対パス
- **Check**: covered(テスト完備), partial(一部不足), missing(テスト無し)
- **Action**: -(何もしない), generated(生成した)

次工程は `Action=generated` の行を処理対象とする。

## サブエージェントへの指示

### 調査フェーズ

```
対象specの調査タスク

## 入力
- specファイル: {絶対パス}
- testsディレクトリ: {code/tauri/src/ および tests/integration/ の絶対パス}

## 手順
1. specをReadで読み込む
2. Format、Source、主要シナリオを特定
3. code/tauri/src/ 内のインラインテスト および tests/integration/ で関連テストを検索:
   - Grepでモジュール名・関数名を検索
   - 見つかったファイルを読んで確認
4. カバレッジを判定:
   - covered: 主要シナリオがすべてテスト済み
   - partial: 一部未テスト（何が不足か明記）
   - missing: 対応test無し

## 出力
Writeで以下に出力:
{tests_dir}/.tests-from-spec/analysis/{spec相対パス}.md

フォーマット:
```markdown
# {spec相対パス}

## spec概要
- Format: Gherkin/StateMachine/Contract/DecisionTable
- Source: {コードファイル}
- シナリオ/状態/API数: X

## 対応test
- ファイル: ...（あれば）
- カバレッジ: covered/partial/missing

## 不足しているテスト（partialの場合）
- シナリオA: 未テスト
- 状態遷移B: 未テスト

## 推奨テスト構造（missingの場合）
- mod + #[test] 構造の提案
- 必要なテストケース一覧
```

## 戻り値
`covered:{spec相対パス}` または `partial:{spec相対パス}:{不足内容}` または `missing:{spec相対パス}`
```

### 生成フェーズ（新規test用）

```
test生成タスク

## 入力
- specファイル: {絶対パス}
- テストテンプレート: {code/tauri/src/内の既存インラインテストを参考}

## 手順
1. specをReadで読み込む
2. 既存テスト（例: code/tauri/src/timer.rs の #[cfg(test)] mod tests）を読んでスタイルを確認
3. specのFormatに応じてテストを生成:
   - Gherkin: Scenarioごとに #[test] fn
   - StateMachine: 状態遷移ごとに mod + #[test]
   - Contract: メソッドごとに mod + #[test]
   - DecisionTable: 行ごとに #[test] fn
4. ヘッダコメントを含める

## 出力
Writeで以下に出力:
{tests_dir}/.tests-from-spec/generated/{カテゴリ}/{ファイル名}_tests.rs

## 戻り値
`generated:{test相対パス}:{ケース数}`
```

### 生成フェーズ（追記用test用）

```
test追記生成タスク

## 入力
- specファイル: {絶対パス}
- 既存test: {既存testの絶対パス}
- 不足内容: {調査フェーズで特定した不足内容}

## 手順
1. specをReadで読み込む
2. 既存testをReadで読み込み、構造を確認
3. 不足しているテストケースのみを生成
4. コメントに `// Supplement for: {既存testパス}` を記載
5. 既存の mod tests 構造に合わせる

## 出力
Writeで以下に出力:
{tests_dir}/.tests-from-spec/generated/{カテゴリ}/{ファイル名}_supplement_tests.rs

## 戻り値
`generated:{test相対パス}:{ケース数}`
```

## 注意事項

- **ソースコードの中身は読まない**: specが唯一の情報源。テストはspecの記述のみから生成する。ただし、**specのメタデータ（Source, Runtime）は必ず活用**してテスト方式を決定すること
- **Runtime に応じた出力**: Rust テストは `tests/.tests-from-spec/generated/` に一時配置。適用時に `code/tauri/src/` 内の対応ファイルにインライン追記、または `tests/integration/` に配置
- **出力先は固定**: 必ず `tests/.tests-from-spec/` に出力する。命名規則は `.<target>-from-<source>/`
- **確認フロー必須**: 生成testは `.tests-from-spec/generated/` に出力し、ユーザーに概要を提示して許可を得てから適用（Step 5参照）
- **partial必須処理**: partial判定のファイルも追記用testを生成する（missingだけでなく）
- **既存スタイル踏襲**: 生成するtestは既存テストのスタイルに合わせる
- **追記用の区別**: 追記用testは `_supplement_tests.rs` サフィックス + コメントで区別
