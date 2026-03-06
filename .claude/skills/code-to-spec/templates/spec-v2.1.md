---
作成日: {YYYY-MM-DD}
更新日: {YYYY-MM-DD}
確認日: -
使用終了日: -
Format: spec-v2.1
---

# Specification: {機能名}

## 0. Meta

| Source | Runtime |
|--------|---------|
| {ソースコードパス} | {Rust / JS-ESM} |

| 項目 | 値 |
|------|-----|
| Related | {関連 spec・ドキュメントパス} |
| Test Type | Unit / Integration / E2E |

### Runtime 定義

| 値 | 意味 |
|----|------|
| Rust | Rust。cargo test でテスト |
| JS-ESM | ES Modules（import/export）。vitest で import してテスト |

### Source テーブルの注意事項

- 1つの spec が複数の Source を持てる（Rust バックエンド + Svelte フロントエンド等）
- Source はコードファイルへの参照。Related はコード以外（spec 間参照、ドキュメント等）
- Runtime は code-to-spec が自動判定する。手動追加時は以下で判定:
  - 拡張子 `.rs` → Rust
  - `export`/`import` 文あり `.ts`/`.svelte` → JS-ESM

## 1. Contract (TypeScript)

> AI Instruction: この型定義を唯一の正解として扱い、モックやテストの型に使用すること。

```typescript
interface Input {
  // 入力の型定義
}

interface Output {
  // 出力の型定義
}
```

## 2. State (Mermaid)

> AI Instruction: この遷移図の全パス（Success/Failure/Edge）を網羅するテストを生成すること。

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: action
    Processing --> Completed: success
    Processing --> Error: failure
    Error --> Idle: reset
    Completed --> [*]
```

## 3. Logic (Decision Table)

> AI Instruction: 各行を test.each のパラメータとしてUnit Testを生成すること。Runtime に応じたテストランナーを使用すること。

| Case ID | Input | Expected | Notes |
|---------|-------|----------|-------|
| UT-01 | ... | ... | 基本ケース |
| UT-02 | ... | ... | 境界値 |
| EX-01 | ... | Throw Error | 異常系 |

## 4. Side Effects (Integration)

> AI Instruction: 結合テストでは以下の副作用をスパイ/モックして検証すること。

| 種別 | 内容 |
|------|------|
| Network | {API呼び出し} |
| Store | {状態更新} |
| Navigation | {画面遷移} |

## 5. Notes

- {補足事項}
