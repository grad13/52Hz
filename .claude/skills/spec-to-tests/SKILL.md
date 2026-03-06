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
7. **適用**: 許可後に `tests/unit/` へ配置
8. **クリーンアップ**: 作業ディレクトリ削除（summaryは残す）
9. **Post**: git commit & tag（完了記録）

## 概要

| 項目 | 内容 |
|------|------|
| 入力 | specディレクトリ or `--from-spec-from-code` |
| 出力 | `tests/.tests-from-spec/` → 確認後 `tests/unit/`（JS）or `tests/php/unit/`（PHP） |
| 形式 | Runtime に応じて vitest（JS）or PHPUnit（PHP） |

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
   - `tests/unit/` 内で関連するテストを検索
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
3. `tests/.tests-from-spec/generated/{カテゴリ}/{ファイル名}.test.js` に出力

#### partialの場合（追記用テスト）

1. 既存テストの構造を確認
2. 不足しているテストケースのみを生成
3. `tests/.tests-from-spec/generated/{カテゴリ}/{ファイル名}-supplement.test.js` に出力
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

1. **新規test**: `.tests-from-spec/generated/` から `tests/unit/` にコピー
2. **追記用test**: 既存テストにマージ（describe内に追記）

### Step 7: git commit

適用後、変更をコミット:

```bash
git add tests/unit/ tests/.tests-from-spec/summary.md
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
│       ├── panel-assemble/
│       │   └── api.test.js              # 新規test
│       └── auth/
│           └── overlay-supplement.test.js  # 追記用test
└── unit/                        # 既存test + 適用後のtest
    └── ...
```

## テスト生成ルール

### Runtime に応じたテスト生成

spec の Meta > Source テーブルから Runtime を読み取り、テスト方式を分岐する。

| Runtime | テストランナー | import 方式 | 出力先 |
|---------|--------------|------------|--------|
| `JS-ESM` | vitest | `import { ... } from 'source.js'` | `tests/.tests-from-spec/generated/{カテゴリ}/{name}.test.ts` |
| `JS-CJS` | vitest | `const { ... } = require('source.js')` | `tests/.tests-from-spec/generated/{カテゴリ}/{name}.test.ts` |
| `JS-IIFE` | — | **テスト生成を保留** | サマリーに「ESM 分離が必要」と報告 |
| `PHP` | PHPUnit | `require_once 'source.php'` | `tests/.tests-from-spec/generated/{カテゴリ}/{Name}Test.php` |
| 未記載 | — | Source の拡張子から推定 | 推定不可なら報告して保留 |

**1つの spec に複数の Runtime がある場合**: Runtime ごとに別のテストファイルを生成する。

**Runtime 未記載時の推定ルール**:
1. 拡張子が `.php` → `PHP`
2. パスに `assets/js/` を含む → `JS-IIFE`
3. パスに `js/` を含む `.ts`/`.js` → `JS-ESM`
4. 推定不可 → テスト生成を保留して報告

### JS テストフォーマット（vitest）

```typescript
/**
 * {モジュール名} - {テスト種別}
 *
 * Tests for: {ソースファイル}
 * Spec: {specファイル}
 * Runtime: {JS-ESM / JS-CJS}
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mock } from 'vitest-mock-extended';
import { ModuleName } from '@code/js/path/to/module.js';

describe('ModuleName', () => {
  // テストケース
});
```

### テストのモッキングパターン

テスト生成時は以下のパターンに従うこと。

#### interface モック: `mock<T>()`

自作 interface（JournalState, JournalStorage, Shell 等）は `vitest-mock-extended` の `mock<T>()` を使う。手書き部分モックは禁止。

```typescript
import { mock } from 'vitest-mock-extended';

// Good: mock<T>() で interface を自動満足
const mockState = mock<JournalState>();
mockState.on.mockImplementation(() => {});

// Good: 部分的にデフォルト値を指定
const mockUtils = mock<JournalContentUtils>({
  isContentEmpty: (c: string) => !c || !c.trim(),
} as Partial<JournalContentUtils>);

// Bad: 手書き部分モック（tsc エラーになる）
const mockState = { on: vi.fn(), emit: vi.fn() };
```

#### globalThis: `(globalThis as any).X`

グローバル変数のモック代入は `(globalThis as any)` を使う。

```typescript
(globalThis as any).FFFFFF_AUTH = { isAuthenticated: () => true };
(globalThis as any).fetch = vi.fn().mockResolvedValue({ status: 200, json: async () => ({}) });
```

#### DOM クエリ: ジェネリクス指定

`querySelector` はジェネリクスで型を絞る。`getByPlaceholderText` 等は `as HTMLInputElement` でキャスト。

```typescript
const btn = el.querySelector<HTMLButtonElement>('.btn')!;
const input = screen.getByPlaceholderText('メール') as HTMLInputElement;
```

#### fetch モック: `vi.fn().mockResolvedValue()`

```typescript
(globalThis as any).fetch = vi.fn().mockResolvedValue({
  status: 200,
  ok: true,
  json: async () => ({ success: true }),
});
```

#### 使い分け表

| 対象 | 手法 | 理由 |
|------|------|------|
| 自作 interface | `mock<T>()` | interface 追従が自動 |
| fetch | `vi.fn()` + `(globalThis as any)` | Response 型は巨大で mock<T> は過剰 |
| DOM (CodeMirror等) | `mock<T>()` or ファクトリ | サードパーティ型の規模による |
| localStorage | `vi.fn()` ベース | jsdom 提供のものを活用 |

### PHP テストフォーマット（PHPUnit）

```php
<?php
/**
 * {モジュール名} - {テスト種別}
 *
 * Tests for: {ソースファイル}
 * Spec: {specファイル}
 * Runtime: PHP
 */

use PHPUnit\Framework\TestCase;

class {ModuleName}Test extends TestCase
{
    // テストケース
}
```

### Format別のテスト生成方針

| Format | テスト構造 | テスト内容 |
|--------|-----------|-----------|
| Gherkin | Scenarioごとにit() / testメソッド | Given/When/Then を検証 |
| StateMachine | 状態遷移ごとにdescribe() / テストグループ | 各遷移の前提条件・アクション・結果 |
| Contract | メソッドごとにdescribe() / テストグループ | 入力・出力・エラー条件 |
| DecisionTable | 行ごとにit() / testメソッド | 入力組み合わせ → 期待出力 |

### テストケース命名

```javascript
// JS (vitest)
// 番号付き: {セクション}-{サブセクション}-{ケース}: 説明
it('1-1-1: Initial mode is blank', () => { ... });

// シナリオベース: Scenario名をそのまま使用
it('Scenario: メニューを開く', () => { ... });
```

```php
// PHP (PHPUnit)
// 番号付き
public function testUT01_InitialModeIsBlank(): void { ... }

// シナリオベース
public function testScenario_OpenMenu(): void { ... }
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

- **Test**: tests/unit/ からの相対パス
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
- testsディレクトリ: {tests/unit/の絶対パス}

## 手順
1. specをReadで読み込む
2. Format、Source、主要シナリオを特定
3. tests/unit/ 内で関連テストを検索:
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
- describe構造の提案
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
- テストテンプレート: {tests/unit/内の既存テストを参考}

## 手順
1. specをReadで読み込む
2. 既存テストを1つ読んでスタイルを確認
3. specのFormatに応じてテストを生成:
   - Gherkin: Scenarioごとにit()
   - StateMachine: 状態遷移ごとにdescribe()
   - Contract: メソッドごとにdescribe()
   - DecisionTable: 行ごとにit()
4. ヘッダコメントを含める

## 出力
Writeで以下に出力:
{tests_dir}/.tests-from-spec/generated/{カテゴリ}/{ファイル名}.test.js

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
5. 既存のdescribe構造に合わせる

## 出力
Writeで以下に出力:
{tests_dir}/.tests-from-spec/generated/{カテゴリ}/{ファイル名}-supplement.test.js

## 戻り値
`generated:{test相対パス}:{ケース数}`
```

## 注意事項

- **ソースコードの中身は読まない**: specが唯一の情報源。テストはspecの記述のみから生成する。ただし、**specのメタデータ（Source, Runtime）は必ず活用**してテスト方式を決定すること
- **Runtime に応じた出力**: JS テストは `tests/.tests-from-spec/generated/` に、PHP テストも同ディレクトリに配置。適用時に `tests/unit/`（JS）or `tests/php/unit/`（PHP）に移動
- **JS-IIFE は保留**: Runtime が `JS-IIFE` の Source はテスト生成を保留し、「ESM 分離が必要」とサマリーに報告する。自己充足型テスト（ソースロジックの再実装）は生成しない
- **出力先は固定**: 必ず `tests/.tests-from-spec/` に出力する。命名規則は `.<target>-from-<source>/`
- **確認フロー必須**: 生成testは `.tests-from-spec/generated/` に出力し、ユーザーに概要を提示して許可を得てから適用（Step 5参照）
- **partial必須処理**: partial判定のファイルも追記用testを生成する（missingだけでなく）
- **既存スタイル踏襲**: 生成するtestは既存テストのスタイルに合わせる
- **追記用の区別**: 追記用testは `-supplement.test.{ts,php}` サフィックス + コメントで区別
