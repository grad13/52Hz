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
4. **生成+適用**: サブエージェント並列でテスト生成 → `tests/unit/` に直接配置（1 code = 1 agent）
5. **テスト実行**: `vitest run`（生成テスト含む全テスト）
6. **サマリー**: 統計・一覧・実行結果を `tests/.tests-from-code/summary.md` に出力
7. **ユーザー確認**: 概要提示 → 判断（commit / 修正 / revert）
8. **クリーンアップ**: 作業ディレクトリ削除
9. **Post**: git commit & tag（完了記録）

## 概要

| 項目 | 内容 |
|------|------|
| 入力 | コードディレクトリ/ファイル or `--from-refactor-code` or `--from-tests-to-spec` |
| 出力 | `tests/.tests-from-code/` （分析）+ `tests/unit/` （テスト直接配置） |
| 一次情報源 | コードファイル |
| 補助情報源 | 既存テスト（パターン・import・モック手法の参照） |
| テストランナー | vitest |

## 位置づけ

3側面一致原則（code ↔ test ↔ spec）の6辺のうち、code → tests の直接辺。

| スキル | 起点 | 終点 | 一次情報源 | 問い |
|--------|------|------|-----------|------|
| spec-to-tests | spec | tests | spec | 「spec の仕様にテストがあるか？」 |
| tests-to-code | tests | code | tests | 「テストが検証する内容とコードが一致するか？」 |
| **code-to-tests** | **code** | **tests** | **code** | **「コードの公開 IF にテストがあるか？」** |

### spec-to-tests との違い

- `spec-to-tests`: spec が正。spec に書いてあるのにテストがない → 生成
- `code-to-tests`: code が正。code に export があるのにテストがない → 検出/生成

spec が存在しない・古い・不完全な場合でも、code から直接テストカバレッジを検証できる。

### spec を使わない

spec は補助情報源として使わない。code のみから分析・生成する。
spec を介在させたい場合は `code-to-spec` → `spec-to-tests` のチェーンを使う。

## 入力オプション

### オプションA: ディレクトリ/ファイル指定

```
code-to-tests code/js/shell
code-to-tests code/js/journal/task/parser.ts
```

指定パス配下のコードファイル（`*.ts`, `*.js`）を対象とする。

### オプションB: refactor-codeの結果を使用

```
code-to-tests --from-refactor-code
```

`code/.refactor/` の結果から対象コードファイルを取得。

### オプションC: tests-to-specの結果を使用

```
code-to-tests --from-tests-to-spec
```

`documents/.spec-from-tests/summary.md` の Source 列からコードファイルを取得。

## 分類

### ファイルレベル分類

| 分類 | 条件 | アクション |
|------|------|-----------|
| **covered** | テストが存在し、公開 IF を網羅 | スキップ（サマリーに記録） |
| **partial** | テストが存在するが、未カバーの公開 IF がある | supplement テスト生成 |
| **missing** | テストファイルが存在しない | フルテスト生成 |

### メソッドレベル分類（partial 内の詳細）

各公開 IF に対して:

| Level | 名前 | 条件 | 生成優先度 |
|-------|------|------|-----------|
| **L1** | untested | テスト自体がない（メソッド名がテストに登場しない） | 高 |
| **L2** | undertested | テストはあるが主要分岐が未カバー | 中 |
| - | tested | 公開 IF がテストされており、主要分岐もカバー | - |

L1 は L2 より深刻。L1 のメソッドが残っている限りファイルは partial。
全メソッドが tested になればファイルは covered。

**L2 判定の基準**（コードの分岐構造から判定）:

- 早期 return（ガード節）がテストされていない
- if/else の片方のみテスト
- エラーパス（throw, console.error）が未テスト
- ループの 0回/1回/N回 のうち一部のみ

### 「公開 IF」の定義

code-to-tests が検査する対象:

1. **export された関数/クラス/定数**
2. **ファクトリ関数が返すオブジェクトの公開メソッド**（`_` プレフィックスを除く）
3. **イベントハンドラ**（on/off/emit のイベント名）

検査**しない**もの:
- `_` プレフィックスの private メソッド（間接テスト対象）
- 型定義のみのファイル（`.d.ts`）
- CSS, データファイル

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
- 指定パス内の `*.ts`, `*.js` を収集
- 除外: `_archive/`, `_prototypes/`, `_exemplars/`, `_tools/`, `node_modules/`, `.d.ts`, CSS, データファイル

**--from-refactor-codeの場合:**
- `code/.refactor/` の結果から対象ファイルリストを取得

**--from-tests-to-specの場合:**
- `documents/.spec-from-tests/summary.md` を読み込み
- Source 列からコードファイルを取得

### Step 2: 分析（1ファイル = 1サブエージェント、並列実行）

各コードファイルについて:

1. **コードファイルを読み込み**
   - export された関数/クラス/定数を抽出
   - ファクトリ関数が返すオブジェクトの公開メソッドを抽出
   - イベントハンドラ（on/off/emit）を抽出
   - 各公開 IF の分岐構造を分析

2. **対応テストを検索**
   - テストファイルの `import` 文から逆引き（最も確実）
   - ファイル名パターンマッチ（`shell/core.ts` → `tests/unit/shell/*core*.test.ts`）
   - テストの `Spec:` ヘッダー → spec の `Source:` → コードファイル（間接マッチ）

3. **ファイルレベル判定**
   - テストファイルが見つからない → **missing**
   - テストファイルが見つかった → 各公開 IF のメソッドレベル判定へ

4. **メソッドレベル判定**（テストファイルが存在する場合）
   - 各公開 IF について:
     - メソッド名がテストに登場しない → **L1 (untested)**
     - テストはあるが主要分岐が未カバー → **L2 (undertested)**
     - テスト済み → **tested**
   - L1 or L2 が1つでもあれば → ファイルは **partial**
   - 全て tested → ファイルは **covered**

5. **結果出力**
   - `tests/.tests-from-code/analysis/{カテゴリ}/{ファイル名}.md` に書き出す

### Step 3: 生成 + 適用（partial/missing のみ、1ファイル = 1サブエージェント、並列実行）

covered はスキップ。partial/missing のみ処理する。

#### missing の場合（フルテスト生成）

1. コードの export を全て抽出
2. 各公開メソッドの分岐構造を分析
3. 既存テストファイルからプロジェクトのテストパターンを参照
4. `tests/unit/{category}/{name}.test.ts` に**直接配置**

#### partial の場合（supplement テスト生成）

1. L1/L2 のメソッドのみ対象
2. 既存テストの構造（describe/it 階層、モック手法）を踏襲
3. `tests/unit/{category}/{name}-supplement.test.ts` に**直接配置**
4. ファイル先頭にコメント: `// Supplement: L1/L2 gaps from code-to-tests`

### Step 4: テスト実行

生成テスト含む全テストを実行:

```bash
cd tests && npx vitest run
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
├── .tests-from-code/
│   ├── summary.md              # 全体サマリー（実行結果込み）
│   └── analysis/               # カバレッジ分析結果
│       ├── shell/
│       │   └── core.md
│       └── journal/
│           └── task-parser.md
└── unit/                       # テスト直接配置先
    ├── shell/
    │   └── vitest-core-supplement.test.ts   # partial → supplement
    └── journal/
        └── vitest-task-parser.test.ts       # missing → new
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
| L1 (untested) | N methods |
| L2 (undertested) | N methods |

## テスト実行結果

| 結果 | Count |
|------|-------|
| pass | N files |
| fail | N files |

## 一覧

| Code | Test | Status | L1 | L2 | 生成 | vitest |
|------|------|--------|----|----|------|--------|
| js/shell/core.ts | unit/shell/vitest-core.test.ts | partial | replacePanel, requestTwin | addPanel(max超過) | supplement | pass |
| js/journal/task/parser.ts | - | missing | - | - | new | pass |
| js/journal/task/cache.ts | unit/journal/vitest-cache.test.ts | covered | - | - | - | - |
```

## 分析ファイルフォーマット

```markdown
---
Code: {コード相対パス}
Test: {テスト相対パス or "なし"}
Status: {covered/partial/missing}
L1: {untested メソッド数}
L2: {undertested メソッド数}
---

# {コードファイル名}

## 公開 IF

| # | 名前 | 種別 | Level | 備考 |
|---|------|------|-------|------|
| 1 | createShellCore | factory | tested | - |
| 2 | addPanel | method | L2 | max panels 超過パスが未テスト |
| 3 | replacePanel | method | L1 | テストなし |
| 4 | requestTwin | method | L1 | テストなし |

## L1 詳細（untested）

### replacePanel
- 行: L130-145
- シグネチャ: `replacePanel(position: string, newPanelId: string, options?: PanelOptions): void`
- 分岐: panelDef 未登録(L131)、既存パネルの close(L136)、新パネルの add(L140)
- 推奨テストケース:
  1. 正常置換（左パネルを別パネルに）
  2. 未登録パネル ID → console.error
  3. 同一パネルへの置換

## L2 詳細（undertested）

### addPanel
- テスト箇所: vitest-core.test.ts L45-80
- テスト済み分岐: 正常追加（左）、正常追加（右）
- 未テスト分岐:
  1. `this.state.panels.length >= 2` → console.warn (L64-67)
  2. 同一 position に既存パネル → console.warn (L69-71)
```

## テスト生成ルール

### テストランナー・モック

| 項目 | 方針 |
|------|------|
| 一次情報源 | コードファイル |
| 補助情報源 | 既存テスト（パターン・import・モック手法の参照） |
| テストランナー | vitest |
| モック | `mock<T>()`（vitest-mock-extended） |
| import | `@code` エイリアス |
| 命名 | 既存テストの命名パターンに従う |

### 期待値導出の原則

コードが一次情報源である以上、テストの期待値はコードのロジックから**厳密に導出**しなければならない。推測・仮定は禁止。

**ロジックトレース義務**:
- 分岐・ループを含む関数のテストでは、具体的な入力値でコードを1行ずつステップ実行し、期待値を導出する
- 「こうなるはず」という推測ではなく、コードの各行を辿った結果をコメントに記録する
- テストコードに**導出過程をコメントで明記**する（レビュー可能にするため）

```typescript
// BAD: 推測で期待値を決定
expect(state._drag.insertIndex).toBe(0);

// GOOD: コードをトレースして期待値を導出し、過程をコメントに残す
// dragIndex=0 → loop: i=0 skip(dragIndex), i=1 center=60, 30>60=false
// → insertIndex stays -1
expect(state._drag.insertIndex).toBe(-1);
```

**イベントハンドラのテスト**:
- ハンドラ内部で呼ばれる関数（renderTable, loadTasks 等）の副作用まで追跡する
- ハンドラが state を直接変更するのか、別の関数経由で変更するのかをコードから確認する

### mock 状態の管理

`mock<T>()` の `.mock.calls` は `vi.restoreAllMocks()` ではクリアされない（spy のみ対象）。テスト間で呼び出し記録が蓄積する。

**ルール**:
- `.mock.calls` からハンドラ・コールバックを取得するテストでは、`beforeEach` で対象の mock を `mockClear()` する
- setup() が何度も呼ばれる場合、`find()` は最初の（古い）呼び出しを返す。これは**別の state オブジェクト**のハンドラである可能性が高い

```typescript
// BAD: 蓄積した mock.calls から find() で取得 → 古いハンドラを返す
const handler = mockObj.on.mock.calls.find(c => c[0] === 'event')[1];

// GOOD: beforeEach で mockClear して最新のハンドラのみにする
beforeEach(() => { mockObj.on.mockClear(); });
// ... setup() ...
const handler = mockObj.on.mock.calls.find(c => c[0] === 'event')[1];
```

### L1 生成（untested メソッド）

公開 IF の正常系 + 全ガード節 + 主要エラーパスをテスト:

```typescript
// テスト生成例: L1 (untested) メソッド
describe('replacePanel', () => {
  it('replaces left panel with new panel', () => { /* 正常系 */ });
  it('logs error for unregistered panelId', () => { /* ガード節 */ });
  it('replaces with same panel id', () => { /* エッジケース */ });
});
```

### L2 生成（undertested メソッド）

分析で特定された未テスト分岐のみをテスト:

```typescript
// テスト生成例: L2 (undertested) メソッド
// Supplement: L1/L2 gaps from code-to-tests
describe('addPanel - supplement', () => {
  it('warns when max panels exceeded', () => { /* 未テスト分岐 */ });
  it('warns when position already occupied', () => { /* 未テスト分岐 */ });
});
```

### テストのモッキングパターン

| 対象 | 手法 | 理由 |
|------|------|------|
| 自作 interface | `mock<T>()` | interface 変更を自動追従 |
| fetch | `vi.fn()` + `(globalThis as any)` | Response 型は巨大で mock<T> は過剰 |
| DOM (CodeMirror等) | `mock<T>()` or ファクトリ | サードパーティ型の規模による |
| localStorage | `vi.fn()` ベース | jsdom 提供のものを活用 |

**禁止**: 手書き部分モック。`mock<T>()` を使う。

### テストファイル命名

| ケース | 命名パターン |
|--------|------------|
| missing → 新規 | `tests/unit/{category}/vitest-{name}.test.ts` |
| partial → supplement | `tests/unit/{category}/vitest-{name}-supplement.test.ts` |

### テストヘッダー

```typescript
// @created: {YYYY-MM-DD}
// @updated: {YYYY-MM-DD}
// @checked: -
/**
 * {モジュール名} - {テスト種別}
 *
 * Tests for: {ソースファイル}
 * Generated by: code-to-tests
 */
```

## テスト対応関係の検出

コードファイルから対応するテストファイルを以下の順序で探索:

1. **import 逆引き**（最も確実）: テストファイルの `import` 文でコードファイルを参照しているものを検索
2. **ファイル名パターンマッチ**: `shell/core.ts` → `tests/unit/shell/*core*.test.ts`
3. **間接マッチ**: テストの `Spec:` ヘッダー → spec の `Source:` → コードファイル

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
- testsディレクトリ: {tests/unit/の絶対パス}

## 手順
1. コードファイルをReadで読み込む
2. 公開IFを抽出:
   - export された関数/クラス/定数
   - ファクトリ関数が返すオブジェクトの公開メソッド（_ プレフィックス除く）
   - イベントハンドラ（on/off/emit のイベント名）
3. 各公開IFの分岐構造を分析:
   - ガード節（早期return）
   - if/else 分岐
   - エラーパス（throw, console.error/warn）
   - ループ
4. 対応テストを検索:
   a. tests/unit/ 内で Grep（import文にコードファイルパスを含むもの）
   b. ファイル名パターンマッチ
   c. 見つかったテストファイルをReadで読み込む
5. ファイルレベル判定:
   - テストなし → missing
   - テストあり → 各メソッドのLevel判定へ
6. メソッドレベル判定（テストがある場合）:
   - メソッド名がテストに登場しない → L1 (untested)
   - テストはあるが主要分岐が未カバー → L2 (undertested)
   - 十分にテスト済み → tested
7. ファイル判定:
   - L1 or L2 が1つでもあれば → partial
   - 全て tested → covered

## 出力
Writeで以下に出力:
{tests_dir}/.tests-from-code/analysis/{カテゴリ}/{ファイル名}.md

フォーマット:
---
Code: {コード相対パス}
Test: {テスト相対パス or "なし"}
Status: {covered/partial/missing}
L1: {untested メソッド数}
L2: {undertested メソッド数}
---

# {コードファイル名}

## 公開 IF

| # | 名前 | 種別 | Level | 備考 |
|---|------|------|-------|------|
| ... |

## L1 詳細（untested）
（各L1メソッドの行範囲、シグネチャ、分岐、推奨テストケース）

## L2 詳細（undertested）
（各L2メソッドのテスト箇所、テスト済み分岐、未テスト分岐）

## 戻り値
`{Status}:{コード相対パス}:{L1数}:{L2数}` または `covered:{コード相対パス}`
```

### 生成フェーズ（missing: フルテスト）

```
テスト生成タスク（missing）

## 入力
- コードファイル: {絶対パス}
- 分析結果: {分析ファイルの絶対パス}
- 参考テスト: {同カテゴリの既存テスト絶対パス}（パターン参照用）

## 手順
1. コードファイルをReadで読み込む
2. 分析結果をReadで読み込む
3. 参考テストをReadで読み込む（スタイル・モック手法の参照）
4. 全公開IFのテストを生成:
   - 正常系
   - 全ガード節
   - 主要エラーパス
5. テストファイルを直接配置:
   tests/unit/{category}/vitest-{name}.test.ts

## テストファイルの要件
- ヘッダー: @created, @updated, @checked アノテーション + JSDocコメント
- import: vitest (describe, it, expect, beforeEach, vi) + vitest-mock-extended (mock) + @code エイリアス
- モック: mock<T>() を使用、手書き部分モック禁止
- 命名: 既存テストの命名パターンに従う

## 期待値導出（必須）
テストの各 expect() に対して、コードのロジックをトレースして期待値を導出すること。
- 分岐・ループは具体値で展開し、結果をコメントに記録する
- 推測で期待値を決めてはならない。コードの行番号を辿って導出する
- イベントハンドラは内部で呼ばれる関数の副作用まで追跡する

## mock 状態の管理（必須）
mock<T>() の .mock.calls はテスト間で蓄積する（vi.restoreAllMocks では消えない）。
- .mock.calls からハンドラを取得する describe では beforeEach で mockClear() する
- setup() を複数テストで呼ぶ場合、find() が古いハンドラを返す問題に注意

## 出力
Writeで tests/unit/{category}/vitest-{name}.test.ts に直接書き出す

## 戻り値
`generated:{テスト相対パス}:{ケース数}`
```

### 生成フェーズ（partial: supplement テスト）

```
テスト生成タスク（partial supplement）

## 入力
- コードファイル: {絶対パス}
- 既存テスト: {既存テストの絶対パス}
- 分析結果: {分析ファイルの絶対パス}

## 手順
1. コードファイルをReadで読み込む
2. 既存テストをReadで読み込み、構造・スタイルを確認
3. 分析結果をReadで読み込む
4. L1/L2 のメソッドのみ対象にテストを生成:
   - L1: 正常系 + 全ガード節 + 主要エラーパス
   - L2: 分析で特定された未テスト分岐のみ
5. 既存テストの構造（describe/it 階層、モック手法）を踏襲
6. テストファイルを直接配置:
   tests/unit/{category}/vitest-{name}-supplement.test.ts
7. ファイル先頭にコメント: // Supplement: L1/L2 gaps from code-to-tests

## 期待値導出（必須）
テストの各 expect() に対して、コードのロジックをトレースして期待値を導出すること。
- 分岐・ループは具体値で展開し、結果をコメントに記録する
- 推測で期待値を決めてはならない。コードの行番号を辿って導出する
- イベントハンドラは内部で呼ばれる関数の副作用まで追跡する

## mock 状態の管理（必須）
mock<T>() の .mock.calls はテスト間で蓄積する（vi.restoreAllMocks では消えない）。
- .mock.calls からハンドラを取得する describe では beforeEach で mockClear() する
- setup() を複数テストで呼ぶ場合、find() が古いハンドラを返す問題に注意

## 出力
Writeで tests/unit/{category}/vitest-{name}-supplement.test.ts に直接書き出す

## 戻り値
`generated:{テスト相対パス}:{ケース数}`
```

## 除外対象

- `_archive/` - アーカイブ済みコード
- `_prototypes/` - プロトタイプ
- `_exemplars/` - リファレンス
- `_tools/` - 開発ツール
- `node_modules/`
- `.d.ts` ファイル（型定義のみ）
- CSS, データファイル（`.json`, `.csv` 等）

## 注意事項

- **コードが一次情報源**: テストはコードファイルから導出する。spec は参照しない
- **spec を使わない**: spec を介在させたい場合は `code-to-spec` → `spec-to-tests` のチェーンを使う
- **直接配置**: 生成テストは `tests/unit/` に直接配置する。`generated/` ステージングは使わない
- **テスト実行必須**: 配置後に vitest を実行し、結果をサマリーに記録する
- **ロールバック**: Pre の git tag でロールバック可能
- **既存スタイル踏襲**: 生成するテストは既存テストのスタイル・モック手法に合わせる
- **mock<T>() 必須**: interface モックは `vitest-mock-extended` の `mock<T>()` を使用。手書き部分モック禁止
- **@code エイリアス**: import は `@code` エイリアスを使用
- **L1 優先**: L1（untested）は L2（undertested）より深刻。L1 を優先的に対処する
- **supplement の区別**: supplement テストは `-supplement.test.ts` サフィックス + コメントで区別
- **クリーンアップは許可後**: `.tests-from-code/` の削除はユーザーの許可を得てから実行する
- **期待値は推測禁止**: テストの expect() の値はコードをトレースして導出する。分岐・ループは具体値で展開し、導出過程をコメントに残す
- **mock.calls の蓄積**: `mock<T>()` の `.mock.calls` は `vi.restoreAllMocks()` ではクリアされない。ハンドラ取得テストでは `beforeEach` で `mockClear()` すること
