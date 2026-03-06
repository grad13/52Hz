---
name: refactor-spec
description: specファイルのリファクタリング候補を分析するスキル。問題の本質と推奨アクションを明確化する。
---

# Refactor Spec

specファイルのリファクタリング候補を分析するスキル。問題の本質と推奨アクションを明確化する。

## クイックリファレンス

1. **Pre**: git commit & tag（ロールバックポイント作成）
2. **対象選定**: `documents/spec/` の `*.md` を収集
3. **対象決定**: 500行超 → must候補、残りは更新日が古い順、合計20件
4. **分析**: サブエージェント並列でファイル分析（1 file = 1 agent、最大20件）
5. **サマリー**: `documents/.refactor-spec/summary.md` に出力
6. **Post**: git commit & tag（完了記録）

※ 分析のみ。実際の修正は推奨アクションに記載されたスキルで実施。

## スコープ

**spec 単体の内部品質のみを検査する。** コードやテストは参照しない。

refactor-* スキル群は全て「単体の内部品質分析」に統一されている:

| スキル | 見るもの | 見ないもの |
|--------|----------|-----------|
| refactor-tests | テストファイル単体 | コード、spec |
| refactor-code | コードファイル単体 | テスト、spec |
| refactor-spec | specファイル単体 | コード、テスト |

コード・テストとの cross-reference は方向スキルが担当する:

| 検査内容 | 担当スキル | 理由 |
|----------|-----------|------|
| specがコードより古い | code-to-spec | コードを見ないと判定できない |
| specに対応テストがない | spec-to-tests | テストを見ないと判定できない |
| specがコードの公開IFを網羅していない | code-to-spec | コードを見ないと判定できない |

## 原則

- **spec-v2.1形式が標準**: Legacy形式は移行対象
- **Sourceテーブル必須**: コードとの紐付けがないspecは不完全
- **推奨アクション明示**: 問題点ごとにどのスキルで対処すべきか提示
- **spec単体で完結**: コード・テストの読み込み禁止

## 判定基準

| 判定 | ID | 条件 | 推奨アクション |
|------|----|------|---------------|
| **must** | M-SIZE | 500行超 | 手動分割 |
| **must** | M-LEGACY | Format が `spec-v2.1` でない | update-spec |
| **must** | M-NO-SRC | Source テーブルが空（パスの正しさは検証しない） | 手動追記 |
| **should** | S-AMBIGUOUS | Decision Tableの空セル、到達不能状態、セクション番号重複、仕様矛盾 | 手動修正 |
| **should** | S-UNMERGED | Supplement セクションが本文に未統合で蓄積 | 手動統合 |

**5項目のみ。** 全てspec単体で判定可能。

## 問題タイプ別の分析要件

判定だけでは不十分。各問題について**そのまま対処に移れるレベル**の分析を出力する。

### M-SIZE: 分割案の構築が必須

判定だけでなく、**具体的な分割案**を出力する。

**必須出力:**
1. **セクション構造**: 全セクションの名前・行範囲・行数
2. **分割案**: 分割後のファイル名・含めるセクション・推定行数
3. **依存関係**: 分割後のファイル間で参照が必要な箇所

```
BAD（判定だけ）:
  503行で500行超。分割が必要。

GOOD（対処可能）:
  セクション構造:
    ## 0. Meta (11-30行, 20行)
    ## 1. Contract (32-180行, 149行)
    ## 2. State (182-250行, 69行)
    ## 3. Logic (252-400行, 149行)
    ## 4. SideEffects (402-450行, 49行)
    ## 5. Notes (452-503行, 52行)

  分割案:
    calendar-panel.md (250行): Meta + Contract
    calendar-panel-behavior.md (270行): State + Logic + SideEffects + Notes
```

### M-NO-SRC: ソースファイル候補の推定が必須

Sourceテーブルが空でも、spec本文にはモジュール名・関数名・パスのヒントがある。

**必須出力:**
1. **spec内の手がかり**: 本文に登場するモジュール名、関数名、ファイルパスの記述
2. **推定ソース**: 手がかりから推定されるソースファイル候補とRuntime
3. **確信度**: 推定の根拠の強さ（明示的パス記載 > 関数名から推定 > 不明）

```
BAD:
  Sourceテーブルが空。手動追記が必要。

GOOD:
  spec内の手がかり:
    - L45: "createMigration() を export"
    - L78: "data/migration.ts モジュール"
    - L120: "localStorage のスキーマバージョン管理"

  推定ソース:
    | Source | Runtime | 根拠 |
    |--------|---------|------|
    | js/data/migration.ts | JS-ESM | L78 にモジュール名明記 |
```

### M-LEGACY: 現在のFormat値の記録

**必須出力:**
1. 現在の Format 値（またはフィールドなし）
2. 推奨: update-spec スキルで変換

### S-AMBIGUOUS: 矛盾箇所の引用と修正案が必須

**必須出力:**
1. **矛盾箇所の引用**: 行番号付きで対立する記述を並べる
2. **矛盾の内容**: 何と何が矛盾しているか
3. **修正案**: どちらを正とするか、または両方をどう修正するか

```
BAD:
  Decision Tableに曖昧な箇所がある。

GOOD:
  矛盾箇所:
    L55: "INVALID_CONTENT → content不正（フォーマットの問題）"
    L120: "INVALID_CONTENT → content未指定（存在しない）"

  矛盾の内容:
    同一エラーコード INVALID_CONTENT が、Grid APIでは「フォーマット不正」、
    Snapshots APIでは「未指定」と異なる意味で使われている。

  修正案:
    Snapshots API 側を MISSING_CONTENT に変更し、コードの意味を分離する。
```

### S-UNMERGED: マージ方針の提示が必須

**必須出力:**
1. **未統合セクション**: セクション名・行範囲
2. **対応する本文セクション**: どこに統合すべきか
3. **マージ方針**: 追記・置換・統合後の構造

```
BAD:
  Supplementセクションが未統合。

GOOD:
  未統合セクション:
    "Overlay Control" (L200-280, 81行)
    ← Supplement from: timer_tests.rs (L200コメント)

  対応する本文セクション:
    本文に対応セクションなし。Timer とは別責務（Overlay制御）。

  マージ方針:
    独立spec化を推奨: spec/overlay.md に移動。
    timer.md の L200-280 を削除し、Meta に依存関係を追記。
```

## 実行モード

### フォアグラウンド（デフォルト）
```
refactor-spec
```
進捗がリアルタイムで表示される。

### バックグラウンド
```
refactor-spec --background
```
処理全体を1つのbackgroundタスクとして実行。完了通知が届く。
結果は `documents/.refactor-spec/summary.md` を参照。

実装方法:
```
Task:
- subagent_type: "general-purpose"
- run_in_background: true
- prompt: "refactor-specスキルを実行"
```

## ワークフロー

### Step 0: Pre - git commit & tag（ロールバックポイント）

**スキル開始前に現在の状態を保存する。**

```bash
# 未コミットの変更があればコミット
git add -A
git diff --cached --quiet || git commit -m "pre: refactor-spec checkpoint"

# タグを作成（ロールバック用）
git tag refactor-spec-pre-$(date +%Y%m%d-%H%M%S)
```

問題発生時に `git reset --hard refactor-spec-pre-XXXXXXXX` でロールバック可能にする。

### Step 1: 対象選定

`documents/spec/` 配下の `*.md` を Glob で収集する。

**除外対象**:
- `README.md`
- `.format/` 配下
- `.refactor-spec/` 配下（作業ディレクトリ）

### Step 2: 対象決定

1. 500行超を `must` としてリストアップ
2. 残りを更新日が古い順にソート
3. must + 古い順上位 = 合計20ファイル（mustが20以上なら全て）

### Step 3: 分析

**1ファイル = 1サブエージェント**（バッチ化禁止）

**並列実行**: 全ファイルのTaskを1つのメッセージで同時に発行する。

```
各Task設定:
- subagent_type: "general-purpose"
- model: "sonnet"
# run_in_background は指定しない（ハングする問題あり）
# 並列化は1メッセージに複数のTask tool callを含めることで実現する
```

**注意**: `run_in_background: true` はハングするため使用しない。並列化は**1メッセージに複数のTask呼び出しを含める**ことで実現する。

### Step 4: サマリー生成

全サブエージェント完了後、`documents/.refactor-spec/summary.md` を生成。

### Step 5: Post - git commit & tag（完了記録）

**スキル完了後に結果をコミット & タグ付け。**

```bash
git add -A
git commit -m "refactor-spec: analyze {N} files"
git tag refactor-spec-post-$(date +%Y%m%d-%H%M%S)
```

## 出力構造

```
documents/.refactor-spec/
├── must/                              # must判定（必須対象）
│   └── spec/path/to/file.md.md        # 元のパス構造を保持
├── should/                            # should判定（推奨対象）
│   └── spec/path/to/file.md.md
└── summary.md
```

**重要: 出力パスは元のdocuments/以下の構造を保持する。**
- 例: `documents/spec/api/auth.md` → `documents/.refactor-spec/must/spec/api/auth.md.md`

### 1ファイル = 1出力（重複禁止）

**1つのspecファイルに対して、出力ファイルは必ず1つ。** must問題とshould問題が混在する場合も1ファイルにまとめる。

**配置ルール:** 最高重度で配置先を決定する。
- must問題が1つでもあれば → `must/` に配置（should問題も同じファイルに含める）
- should問題のみ → `should/` に配置

**Judgmentフィールド:** 最高重度を記載する。
- must + should混在 → `Judgment: must`
- shouldのみ → `Judgment: should`

```
例: tasks.md に M-SIZE（must）と S-UNMERGED（should）がある場合

出力: must/spec/journal/tasks.md.md （1ファイルのみ）
---
Judgment: must
Issues: [M-SIZE, S-UNMERGED]
---
内容: M-SIZE の分割案 + S-UNMERGED のマージ方針（両方含む）

NG: must/ と should/ の両方にファイルを作成してはならない
```

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

**現状**: {何が起きているか、行番号付きで具体的に}
**本質**: {なぜそれが問題なのか}
**あるべき姿**: {どうあるべきか}
**推奨アクション**: {対処スキル名 or 手動修正}

{問題タイプ別の詳細分析 — 「問題タイプ別の分析要件」セクション参照}
```

**重要: 各問題の分析は「問題タイプ別の分析要件」に定義された必須出力を全て含めること。判定だけの出力は不可。**

## サブエージェントへの指示

**1ファイル = 1サブエージェント**。各サブエージェントは**同一の分析プロセス**を実行する。

### プロンプトテンプレート

```
specファイル分析タスク

対象: {絶対パス}

## 重要: spec 単体の内部品質のみを検査する
コードファイルやテストファイルは一切読まないこと。
spec ファイルだけを Read して判定する。

## 手順
1. Readでspecファイル読込
2. 以下の5項目を判定:
   - M-SIZE: 500行超？ → must
   - M-LEGACY: frontmatter の Format が spec-v2.1 でない？ → must
   - M-NO-SRC: Meta セクションの Source テーブルが空？ → must
   - S-AMBIGUOUS: Decision Tableの空セル、到達不能状態、セクション番号重複、仕様矛盾？ → should
   - S-UNMERGED: Supplement セクションが本文に未統合で蓄積？ → should
   - 該当なし → clean
3. 該当した問題ごとに、以下の「問題タイプ別の分析要件」に従って詳細分析を行う

### Format判定の詳細
frontmatter の `Format:` フィールドを確認する。
- `Format: spec-v2.1` → OK
- それ以外、またはフィールドなし → M-LEGACY

### Sourceテーブル判定の詳細
`## 0. Meta` セクション内に `| Source | Runtime |` テーブルがあるか確認する。
- テーブルにデータ行あり → OK
- ヘッダーのみでデータ行なし、またはテーブル自体なし → M-NO-SRC
※ パスの正しさは検証しない（コードの読み込み禁止）

### S-AMBIGUOUS判定の詳細
spec 内部の整合性を確認する:
- Decision Table に空セル（入力や期待値が未記入）がないか
- State Diagram に到達不能な状態がないか
- セクション番号の重複がないか（`## 0. Meta` が2箇所ある等）
- 同一仕様について矛盾する記述がないか

### S-UNMERGED判定の詳細
ファイル末尾に「Supplement」「追記」「追加内容」等のセクションが
本文に統合されず蓄積していないか確認する。
- Supplement セクションが1つ以上あり、本文の対応セクションと内容が重複/矛盾 → S-UNMERGED
- Supplement がない、または統合済み → OK

## 問題タイプ別の分析要件

判定だけでは不十分。各問題について「そのまま対処に移れるレベル」の分析を出力する。

### M-SIZE の場合（必須出力）
1. セクション構造: 全セクションの名前・行範囲・行数
2. 分割案: 分割後のファイル名・含めるセクション・推定行数
3. 依存関係: 分割後のファイル間で参照が必要な箇所

### M-NO-SRC の場合（必須出力）
1. spec内の手がかり: 本文に登場するモジュール名、関数名、ファイルパスの記述（行番号付き）
2. 推定ソース: 手がかりから推定されるソースファイル候補とRuntime
3. 確信度: 推定の根拠の強さ

### M-LEGACY の場合（必須出力）
1. 現在のFormat値（またはフィールドなし）
2. 推奨: update-spec スキルで変換

### S-AMBIGUOUS の場合（必須出力）
1. 矛盾箇所の引用: 行番号付きで対立する記述を並べる
2. 矛盾の内容: 何と何が矛盾しているか
3. 修正案: どちらを正とするか、または両方をどう修正するか

### S-UNMERGED の場合（必須出力）
1. 未統合セクション: セクション名・行範囲
2. 対応する本文セクション: どこに統合すべきか
3. マージ方針: 追記・置換・独立spec化等

## 禁止事項
- コードファイルの読み込み禁止（Read, Grep, Glob でコードを参照しない）
- テストファイルの読み込み禁止
- Bashツールの使用禁止
- specファイルの修正禁止（分析のみ）

## 出力フォーマット（must/shouldの場合のみ）

以下のフォーマットでWriteツールに出力:

---
File: {相対パス}
Lines: {行数}
Judgment: {must/should}
Issues: [{問題IDリスト}]
---

# {ファイル名}

## 問題点

### 1. [{問題ID}] {問題タイトル}

**現状**: {何が起きているか、行番号付きで具体的に}
**本質**: {なぜそれが問題なのか}
**あるべき姿**: {どうあるべきか}
**推奨アクション**: {対処スキル名 or 手動修正}

{問題タイプ別の必須出力をここに記載}

## 出力先（1ファイル = 1出力、重複禁止）
1つのspecファイルに対して出力は必ず1ファイル。全問題を1ファイルにまとめる。

配置先は最高重度で決定:
- must問題が1つでもあれば → {docs_dir}/.refactor-spec/must/{相対パス}.md
- should問題のみ → {docs_dir}/.refactor-spec/should/{相対パス}.md

**禁止**: must/ と should/ の両方に同じspecの出力を作成してはならない。

## 戻り値
`must:{相対パス}` または `should:{相対パス}` または `clean:{相対パス}`
（must + should 混在の場合は `must:` を返す）
```

### 分析フロー（全ファイル共通）

**重要: spec ファイルのみを読む。コード・テストは一切参照しない。**

```
Phase 1: 判定（5項目チェック）
  1. Readツールでspecファイルを読み込む
  2. frontmatterを確認: Format フィールドの値、更新日
  3. 行数を確認（Readの出力行番号から）
  4. Meta セクションを確認: Source テーブルの有無とデータ行の有無
  5. Decision Table を確認: 空セル、到達不能状態
  6. セクション構造を確認: 番号重複、仕様矛盾
  7. Supplement セクションの確認: 未統合の追記が蓄積していないか

Phase 2: 詳細分析（該当した問題のみ）
  8. 各問題について「問題タイプ別の分析要件」の必須出力を全て作成する:
     - M-SIZE → セクション構造 + 分割案
     - M-NO-SRC → spec内の手がかり + 推定ソース
     - M-LEGACY → 現在のFormat値
     - S-AMBIGUOUS → 矛盾箇所の引用 + 修正案
     - S-UNMERGED → 未統合セクション + マージ方針

Phase 3: 出力（1ファイル = 1出力、重複禁止）
  9. 問題があれば:
     - 全問題（must + should）を1ファイルにまとめる
     - 配置先は最高重度で決定: must問題が1つでもあれば must/、shouldのみなら should/
     - **必ずWriteツールで1ファイルだけ書き出す**
     - 同じspecに対して must/ と should/ の両方にファイルを作成してはならない
  10. 問題なければ:
     - 「clean」としてファイル名を返す（サマリー用）
```

**重要: Phase 2 を省略してはならない。判定だけの出力（「500行超、分割が必要」等）は不可。**

**禁止事項**:
- コードファイルの読み込み（Sourceパスの存在確認も含む）
- テストファイルの読み込み
- Bashツールの使用
- スクリプトの作成・実行
- specファイルの修正（分析のみ）

**重要: 問題があるファイルは、必ずWriteツールでファイルに書き出すこと。**

### 出力

各サブエージェントは分析結果を `documents/.refactor-spec/` に書き出し、**最小限の戻り値**を返す:

```
must:spec/api/auth.md
should:spec/core/parser.md
clean:spec/utils/format.md
```

**パス構造保持**: 出力ファイルは元のdocuments/以下の構造を維持する。
- 入力: `documents/spec/api/auth.md`
- 出力: `documents/.refactor-spec/must/spec/api/auth.md.md`

**コンテキスト節約**: 詳細は全てファイルに書き出す。戻り値は判定と相対パスのみ。

## 除外対象

- `README.md` - ディレクトリ説明
- `.format/` - フォーマット定義
- `.refactor-spec/` - 作業ディレクトリ

## 注意事項

- **spec 単体で完結**: コード・テストは一切読まない。パスの正しさも検証しない
- **バッチ化禁止**: 1ファイル = 1サブエージェント
- **並列実行**: 1メッセージに複数のTask呼び出しを含めて並列化する。run_in_backgroundは使用しない（ハングする問題あり）
- **20ファイル制限**: 1回の実行で最大20ファイル（mustが多ければそれ以上）
- **許可不要**: サブエージェントは `mode="bypassPermissions"` で実行し、ユーザーに許可を求めない
- **1ファイル = 1出力**: 1つのspecに対して出力は1ファイルのみ。must/ と should/ の両方に同じspecの出力を作成してはならない
- **ファイル出力必須**: must/shouldの場合、必ず `documents/.refactor-spec/` にWriteツールでファイルを書き出す
- **パス構造保持**: 出力ファイルは元のdocuments/以下のディレクトリ構造を維持する
- **出力先明示**: プロンプトで絶対パスを明示（サブエージェントが誤った場所に出力する問題を防ぐ）
- **Bash禁止**: Read/Write のみ使用（Grep/Glob も不要 — spec 1ファイルを Read するだけ）
- **分析のみ**: specファイルの修正は行わない
- **推奨アクション必須**: 各問題点に対して、どのスキルで対処すべきか明示する
