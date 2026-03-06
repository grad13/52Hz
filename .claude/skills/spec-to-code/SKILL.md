---
name: spec-to-code
description: spec を真として code との乖離を検出し、**具体的な選択肢と評価を提示してユーザーの判断を仰ぐ**スキル。
---

# Spec to Code

spec を真として code との乖離を検出し、**具体的な選択肢と評価を提示してユーザーの判断を仰ぐ**スキル。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: spec ファイルを収集
3. **分析**: サブエージェント並列で spec↔code 照合（1 spec = 1 agent）
4. **サマリー**: `code/.code-from-spec/summary.md` に出力
5. **ユーザー確認**: 乖離一覧 + 選択肢を提示 → ユーザーが各項目に回答
6. **適用**: ユーザーの回答に従い code または spec を修正
7. **Post**: git commit & tag（完了記録）

## 概要

| 項目 | 内容 |
|------|------|
| 入力 | spec ディレクトリ or `--from-tests-to-spec` / `--from-code-to-spec` |
| 出力 | `code/.code-from-spec/` に分析結果 |
| 原則 | 乖離を検出し選択肢を提示。判断はユーザーに委ねる |
| 情報源 | spec と code の2側面のみ。test は読まない |

## 設計原則

### spec と code の2側面のみを扱う

test は読まない。それは tests-to-code の役割。
spec-to-code は **spec と code を読み比べて乖離を見つけ、具体的な選択肢を提示する**。

### 判断はユーザーに仰ぐ

乖離が見つかったとき「どちらが正しいか」はスキルが決めない。
各乖離について以下を提示する:

1. **乖離の事実**: spec は X と言っている。code は Y をしている。
2. **選択肢**: それぞれ「そのまま実行できるレベル」まで具体化
3. **評価**: 各選択肢の利点・欠点を観点別に評価
4. **推奨**: 明確に優れた選択肢がある場合は推奨を付ける

## 乖離の分類

spec-v2.1 のセクション構造に対応:

| ID | セクション | 乖離の種類 | 例 |
|----|-----------|-----------|-----|
| G1 | 0. Meta | Source パスの不一致 | spec は `timer.rs` だが実装は `timer/` モジュールに分割 |
| G2 | 1. Contract | 公開 IF の不一致 | spec は `advance(&mut self, idle: bool)` だが code は `advance(&mut self)` |
| G3 | 1. Contract | 型の不一致 | Rust: spec は `Option<T>` だが code は `Result<T, E>` / TS: spec は `string` だが code は `number` |
| G4 | 2. State | 状態・初期値の不一致 | Rust: spec は `None` だが code は `Default::default()` / TS: spec は `null` だが code は `undefined` |
| G5 | 3. Logic | 決定表/ルールとの不一致 | spec は条件 X で Y を返すが code は Z を返す |
| G6 | 4. SideEffects | 副作用の不一致 | spec は event `timer-tick` を emit するが code は emit しない |
| G7 | - | spec にあるが code にない | spec が定義する `apply_settings()` が code に存在しない |
| G8 | - | code にあるが spec にない | code の `pub fn` が spec に記載なし |

## 選択肢の評価

### 評価の観点

全観点を網羅する必要はない。その乖離にとって判断材料になるものを選ぶ。

| 観点 | 説明 | 例 |
|------|------|-----|
| 意味の正確さ | 命名・値がドメインの意味を正確に表しているか | `empty` vs `blank` |
| 一貫性 | 既存コード・他モジュールとの命名・パターンの整合 | 他の mode 値との統一 |
| 型安全性 | Rust の型チェック (`cargo check`) や TypeScript の型チェック (`svelte-check`) の恩恵を受けられるか | 型注釈の有無、`Option` vs `unwrap` |
| 互換性 | 変更による既存コード・テストへの影響範囲 | 広く使われている値の変更リスク |
| 拡張性 | 将来の機能追加・変更に対する柔軟性 | interface vs Record<string, any> |
| 簡潔さ | 変更量・複雑さの少なさ | 1行変更 vs 複数ファイル変更 |

### 推奨の付け方

| 状況 | 書き方 |
|------|--------|
| 明確に優れた選択肢がある | **推奨: A** — {理由} |
| 甲乙つけがたい | **判断ポイント**: {何が決め手になるか} |
| ユーザーの方針次第 | **要判断**: A は〜の場合に適切、B は〜の場合に適切 |

## 入力オプション

### オプションA: ディレクトリ指定

```
spec-to-code _documents/spec/shell
spec-to-code _documents/spec/journal
```

指定ディレクトリ内の spec ファイル（`*.md`、Format: spec-v2.1）を対象とする。

### オプションB: tests-to-spec の結果を使用

```
spec-to-code --from-tests-to-spec
```

`_documents/.spec-from-tests/summary.md` から対処済み spec リストを取得。
tests-to-spec で更新/生成された spec を優先的に検証できる。

### オプションC: code-to-spec の結果を使用

```
spec-to-code --from-code-to-spec
```

`_documents/.spec-from-code/summary.md` から生成済み spec リストを取得。

## ワークフロー

### Step 0: Pre - git commit & tag

```bash
git add -A
git diff --cached --quiet || git commit -m "pre: spec-to-code checkpoint"
git tag spec-to-code-pre-$(date +%Y%m%d-%H%M%S)
```

### Step 1: 対象選定

**ディレクトリ指定の場合:**
- 指定ディレクトリ内の `*.md` を収集
- frontmatter に `Format: spec-v2.1` があるものを対象
- Meta > Source テーブルからコードファイルを特定

**--from-tests-to-spec の場合:**
- `_documents/.spec-from-tests/summary.md` を読み込み
- Action が `-` でないもの（generated / updated / mapped）を対象リストに追加

**--from-code-to-spec の場合:**
- `_documents/.spec-from-code/summary.md` を読み込み
- Action が `-` でないものを対象リストに追加

### Step 2: 分析（1 spec = 1 サブエージェント、並列実行）

各 spec ファイルについて:

1. **spec を読み込み**
   - Meta > Source テーブルからコードファイルパスを取得
   - Contract、State、Logic、SideEffects の各セクションを把握

2. **コードファイルを読み込み**
   - 公開 IF（export、関数シグネチャ）を特定
   - 内部状態、ロジック、副作用を把握

3. **セクションごとに照合**
   - G1: Meta.Source のパスは正しいか
   - G2/G3: Contract の公開 IF・型は一致するか
   - G4: State の初期値・遷移は一致するか
   - G5: Logic の決定表/ルールは一致するか
   - G6: SideEffects の副作用は一致するか
   - G7: spec にあって code にないものはないか
   - G8: code にあって spec にないものはないか

4. **乖離ごとに選択肢を具体化**
   - A: code を修正する場合の具体的な変更内容（ファイル、行番号、変更前→変更後）
   - B: spec を修正する場合の具体的な変更内容（セクション、変更前→変更後）
   - C:（必要に応じて第3の選択肢）

5. **各選択肢を評価**
   - 評価観点テーブルから該当するものを選び、短く評価
   - 推奨がある場合は付与

6. **結果出力**
   - 乖離あり: `code/.code-from-spec/analysis/{カテゴリ}/{spec名}.md` に出力
   - 乖離なし: 分析ファイルは省略（summary に match として記録）

### Step 3: サマリー生成

`code/.code-from-spec/summary.md` を生成。

### Step 4: ユーザー確認

乖離一覧を提示し、各項目についてユーザーの判断を仰ぐ。

提示内容:
1. 統計サマリー（match / gap の件数）
2. 各乖離の概要と選択肢
3. 推奨がある場合はその理由

ユーザーは各乖離に対して:
- A/B/C のいずれかを選択
- 「全部推奨通り」等の一括回答も可

### Step 5: 適用（ユーザー回答後）

ユーザーの回答に従い:
- **A（code修正）選択の場合**: コードファイルを Edit で修正
- **B（spec修正）選択の場合**: spec ファイルを Edit で修正
- **C（保留等）選択の場合**: 何もしない（記録のみ）

修正後、spec / code の `更新日` を更新。

### Step 6: Post - git commit & tag

```bash
git add -A
git commit -m "spec-to-code: {N} gaps resolved ({A}A {B}B {C}C)"
git tag spec-to-code-post-$(date +%Y%m%d-%H%M%S)
```

## 出力構造

```
code/
├── .code-from-spec/
│   ├── summary.md
│   └── analysis/
│       ├── timer/timer.md            # 乖離あり: 選択肢付き
│       ├── presence/presence.md     # 乖離あり: 選択肢付き
│       └── ...                      # match は省略
```

## 分析ファイルのフォーマット

```markdown
---
Spec: {spec 相対パス}
Source: {code 相対パス}
Result: {match / gap}
Gaps: {乖離数}
---

# {spec 名}

## 乖離 1: [G5] Logic — Phase 初期値

**spec**: Section 3 — phase 初期値は `Phase::Work`
**code**: `tauri/src/timer.rs` L28 — `phase: Phase::Idle`

### 選択肢

| 選択肢 | 内容 | 評価 |
|--------|------|------|
| **A（code修正）** | L28 を `phase: Phase::Work` に変更 | `Work` はタイマー開始時の意味を正確に表現。他の Phase 値（`Break`, `Done`）との一貫性あり |
| **B（spec修正）** | spec の phase 初期値を `Phase::Idle` に変更 | `Idle` は既存コードの起動フローと整合しており互換性が高い |

**推奨**: A — `Work` の方がタイマーの意味が明確で命名一貫性がある

---

## 乖離 2: [G2] Contract — apply_settings の引数型

**spec**: Section 1.2 — `pub fn apply_settings(&mut self, settings: &Settings)`
**code**: `tauri/src/timer.rs` L85 — `pub fn apply_settings(&mut self, work: u64, short_break: u64)`（個別引数）

### 選択肢

| 選択肢 | 内容 | 評価 |
|--------|------|------|
| **A（code修正）** | `Settings` 構造体を導入し引数を統合 | 型安全性が向上。将来の設定項目追加が容易 |
| **B（spec修正）** | spec を個別引数 `(work: u64, short_break: u64)` に変更 | 現実を反映するが、拡張性が低い |
| **C（保留）** | 設定項目が増えるタイミングでまとめてリファクタ | 現時点で動作影響なし。乖離は残る |

**推奨**: A — 構造体の導入は安全な変更で拡張性が向上する
```

## summary.md フォーマット

```markdown
# Spec to Code Summary

実行日: {YYYY-MM-DD}

## 統計

| Result | Count |
|--------|-------|
| match  | N     |
| gap    | N     |

## 乖離一覧

| # | Spec | Source | ID | 概要 | 推奨 |
|---|------|--------|----|------|------|
| 1 | timer/timer.md | tauri/src/timer.rs | G5 | phase 初期値 Work vs Idle | A（code修正） |
| 2 | ... | ... | ... | ... | ... |

## match 一覧

| Spec | Source |
|------|--------|
| frontend/settings-store.md | frontend/lib/settings-store.ts |
| ... | ... |
```

## サブエージェントへの指示

### 分析フェーズ

```
spec↔code 照合タスク

## 入力
- spec ファイル: {絶対パス}
- code ベースディレクトリ: {code/ の絶対パス}

## 手順
1. spec を Read で読み込む
2. Meta > Source テーブルからコードファイルパスを取得
3. コードファイルを Read で読み込む
4. 以下の順序で照合:

   ### G1: Meta
   - Source パスが実在するか確認

   ### G2/G3: Contract
   - spec の公開 IF（関数名、引数、戻り値、型）
   - code の export / 公開関数のシグネチャ
   - 不一致があれば乖離として記録

   ### G4: State
   - spec の状態定義（初期値、遷移条件）
   - code の内部状態（変数宣言、初期値）
   - 不一致があれば乖離として記録

   ### G5: Logic
   - spec の決定表/ルール（Section 3 の各テーブル行）
   - code の対応するロジック（if文、switch文、条件式）
   - 不一致があれば乖離として記録

   ### G6: SideEffects
   - spec の副作用定義（イベント、DOM操作、API呼び出し）
   - code の実際の副作用
   - 不一致があれば乖離として記録

   ### G7: spec にあるが code にない
   - spec が定義するが code に見つからないメソッド/プロパティ

   ### G8: code にあるが spec にない
   - code の公開 IF で spec に記載がないもの

5. 各乖離について:
   - A（code修正）の具体的な内容（ファイル、行番号、変更前→変更後）
   - B（spec修正）の具体的な内容（セクション、変更前→変更後）
   - C（第3選択肢、必要な場合のみ）
   - 各選択肢の評価（意味の正確さ、一貫性、型安全性、互換性、拡張性、簡潔さ から該当するもの）
   - 推奨（明確な場合のみ）

6. 乖離なしの場合は `match` として返す

## 出力（乖離ありの場合のみ）
Write で以下に出力:
{code_dir}/.code-from-spec/analysis/{カテゴリ}/{spec名}.md

## 戻り値
`match:{spec相対パス}` または `gap:{spec相対パス}:{乖離数}`
```

## 除外対象

- `_archive/` 配下の spec
- frontmatter に `使用終了日` が設定されている spec
- `Format:` が spec-v2.1 以外の spec（旧形式は対象外）

## 注意事項

- **test は読まない**: spec と code の2側面のみを扱う。test は tests-to-code の役割
- **判断はユーザーに仰ぐ**: スキルは乖離を検出し選択肢を提示するまで。修正の判断はユーザー
- **選択肢は具体的に**: 「乖離があります」だけでは不十分。ファイル・行番号・変更内容まで明記
- **評価は根拠付き**: 「Aが良い」ではなく「Aは型安全性が向上し、命名一貫性がある」
- **match は軽く**: 乖離なしのファイルは summary に1行記録するだけ。分析ファイルは不要
- **確認フロー必須**: 適用前に必ずユーザーの回答を得る
- **Format 限定**: spec-v2.1 形式の spec のみ対象。旧形式は refactor-spec で変換後に実行
