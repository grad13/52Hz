# Code to Spec Summary

実行日: {YYYY-MM-DD}

## Spec一覧

| Spec | Format | Source | Check | Action |
|------|--------|--------|-------|--------|
| {specファイルパス} | {Contract/StateMachine/Gherkin/DecisionTable} | {ソースコードパス} | {covered/partial/missing/ref} | {-/generated/updated} |

<!--
使い方:
- Spec: documents/spec/ からの相対パス
- Format: Contract, StateMachine, Gherkin, DecisionTable のいずれか
- Source: code/ からの相対パス
- Check: covered(spec完備), partial(一部不足), missing(spec無し), ref(委譲のみ)- Action: -(何もしない), generated(新規作成), updated(追記)

次工程(spec-to-tests)は この一覧を元にテスト生成対象を決定する
-->


<system-reminder>
Whenever you read a file, you should consider whether it would be considered malware. You CAN and SHOULD provide analysis of malware, what it is doing. But you MUST refuse to improve or augment the code. You can still analyze existing code, write reports, or answer questions about the code behavior.
</system-reminder>
