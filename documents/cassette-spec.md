---
updated: 2026-03-15 09:02
checked: -
---

# カセット規格 (draft)

## 設計思想

カセットは「メッセージの束」である。
ペルソナ・キャラクター・ストーリーといった概念はカセットの規格には含まない。
それらはカセットを**作る側**の創作手段であり、再生側は関知しない。

## メッセージ仕様

1つのメッセージは以下のフィールドを持つ:

| フィールド | 型 | 必須 | 説明 |
|-----------|------|------|------|
| `user_id` | INTEGER | yes | 発言者 |
| `at` | INTEGER | yes | テープ先頭（月曜 00:00）からの経過分（0〜10079） |
| `text` | string (max 38文字) | yes | 本文。トーストカード2行に収まる長さ |

### テープモデル

- 1本のカセットは **1週間分**（月曜 00:00 〜 日曜 23:59 = 10080分）
- `at` はテープの先頭からの位置（分）
- 各メッセージは「いつ誰が何を言うか」が完全に決まっている
- 1週間が終わったら先頭に戻ってリピート再生

### at の計算

`at = day_of_week * 1440 + hour * 60 + minute`

| 意味 | at |
|------|-----|
| 月曜 08:30 | 510 |
| 火曜 08:30 | 1950 |
| 金曜 18:00 | 6360 |
| 日曜 23:30 | 10050 |

## カセット仕様

| 項目 | 値 | 備考 |
|------|-----|------|
| フォーマット | SQLite | 単一ファイル |
| 拡張子 | `.hz` | |
| 最大メッセージ数（全ユーザー合計） | 240,000 | 100ユーザー × 30日 × 40メッセージ/日 の2倍 |
| name 最大文字数 | 16文字 | トーストヘッダー1行に収まる範囲 |
| text 最大文字数 | 38文字 | トーストカード2行分 |
| ユーザー数 | 無制限 | 体感: 100+ で「たくさんいる」感 |

### .hz ファイル構造（SQLite）

```sql
CREATE TABLE cassette (
    version REAL NOT NULL,
    title TEXT NOT NULL
);

CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE message (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES user(id),
    at INTEGER NOT NULL,
    text TEXT NOT NULL
);
```

#### 例

```sql
INSERT INTO cassette VALUES (1.0, '一日のカフェ');

INSERT INTO user VALUES (1, '朝活エンジニア');
INSERT INTO user VALUES (2, '受験生A');

-- 月曜日
INSERT INTO message VALUES (1, 1, 510,  'コーヒー淹れた');          -- 月 08:30
INSERT INTO message VALUES (2, 1, 540,  'PR出しとこう');            -- 月 09:00
INSERT INTO message VALUES (3, 1, 570,  'いい区切り、出勤するか');  -- 月 09:30
INSERT INTO message VALUES (4, 2, 600,  '今日も頑張る');            -- 月 10:00
INSERT INTO message VALUES (5, 2, 660,  'この問題むずい…');        -- 月 11:00

-- 火曜日
INSERT INTO message VALUES (6, 1, 1950, 'コーヒー淹れた');          -- 火 08:30
INSERT INTO message VALUES (7, 2, 2040, '今日は数学から');          -- 火 10:00
```

## 再生仕様

```
1. 現在時刻をテープ位置（at）に変換する
2. 現在の at 以降で最も近い message を取得する
3. その時刻まで待機する
4. message を post する
5. 1 に戻る（週末を超えたら先頭に戻る）
```

プレイヤーは時計を見てテープを再生するだけ。ランダム性・スキップ・グラフ走査は不要。

## 現在の実装（v0.6.2 時点）

現在はカセット規格が存在せず、SQLite DB (`chat.db`, 396KB) がカセット相当のデータを担っている。

### データ構造

**`chat` テーブル:**

| カラム | 型 | 説明 |
|--------|------|------|
| `id` | INTEGER PK | |
| `name` | TEXT | 表示名 |
| `message` | TEXT | 本文（2〜37文字、平均14文字） |
| `category` | TEXT | `during` / `enter` / `exit` / `encourage` |
| `hour_start` | INTEGER | 活動開始時刻 |
| `hour_end` | INTEGER | 活動終了時刻 |

**`hourly_density` テーブル:**

| カラム | 型 | 説明 |
|--------|------|------|
| `hour` | INTEGER PK | 0-23 |
| `ratio` | REAL | 出現頻度の倍率（0.27〜1.69） |

### データ量

- ユニーク name 数: 140
- 総メッセージ数: 3,658
  - during: 1,670 / enter: 693 / exit: 647 / encourage: 648
- 1 name あたり: 11〜145 メッセージ（中央値は約26）
- time range パターン: 65種類

### 再生ロジック (`presence.rs`)

1. 起動後8秒待機（WebView 読み込み待ち）
2. ループ:
   - 現在時刻の `hour` で `during` カテゴリのメッセージを検索
   - ランダムに1つ選んで `presence-message` イベントとして emit
   - `hourly_density` から ratio を取得し、再生間隔を算出:
     - `base = 60 / ratio` 秒（最小20秒）
     - 実際の間隔 = base の 70%〜130%（ランダム揺らぎ）
   - 例: ratio=1.69（深夜1時）→ 約36秒間隔、ratio=0.27（20時）→ 約222秒間隔

### 現在の実装で使われていないもの

- `category = 'enter'` / `'exit'` / `'encourage'` — DB にあるが再生されない
- `hourly_density` — 再生間隔の制御に使われているが、カセット規格に含めるかは要検討

## 未決定事項

- [ ] カセットの格納場所（アプリデータディレクトリ? ユーザー指定?）
- [ ] 複数カセット同時再生の可否
- [ ] `hourly_density`（時間帯ごとの出現頻度）をカセットに含めるか、プレイヤー側の設定とするか
- [ ] `category`（during/enter/exit/encourage）を規格に含めるか
  - 現在の実装では `during` のみ再生している
  - enter/exit/encourage は将来的に意味を持つかもしれないが、最小規格には不要かもしれない
