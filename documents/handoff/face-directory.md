# .face/ ディレクトリ規約

Hub（orbital-q.sakura.ne.jp）にプロジェクトを掲載するための規約。

## 概要

各プロジェクトはサーバー上の自身のディレクトリ内に `.face/` ディレクトリを配置する。Hub はこのディレクトリを走査してプロジェクトカードを生成する。

## ディレクトリ構造

```
52Hz/
├── .face/
│   ├── app.json         # 必須
│   └── thumbnail.png    # 任意
```

## app.json

Hub が読み取るメタデータ。`url` フィールドがあるため、カードクリックで GitHub ページが新しいタブで開く。

```json
{"name": "52Hz", "description": "休憩リマインダーアプリ", "url": "https://github.com/grad13/52Hz"}
```

| フィールド | 必須 | 型 | 用途 |
|-----------|------|-----|------|
| `name` | はい | string | カードのタイトル |
| `description` | いいえ | string | カードの説明文 |
| `url` | いいえ | string | 外部リンク先URL（省略時はディレクトリへの内部リンク） |
| `news` | いいえ | array | 更新情報。各要素は `{"date": "YYYY-MM-DD", "text": "内容"}` |

## thumbnail.png

- 任意。省略時はプロジェクト名をテキスト表示するフォールバックが適用される
- 正方形推奨（aspect-ratio: 1/1 で表示される）
- ファイルサイズは軽量に（表示サイズは最大約 400px 幅）

## デプロイ

自身のデプロイプロセスで `.face/` ディレクトリをサーバーに配置する。

## 対応記録

- 2026-03-13: `.face/app.json` + `thumbnail.png` 作成完了。`.gitignore` に `!.face/` 例外追加。
