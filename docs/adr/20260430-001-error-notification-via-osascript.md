# エラー通知の macOS 通知表示

## Status

Accepted

## Context

`djay-audio-loader-helper` 経由でホットキーから実行する場合、ユーザーはターミナルを見ていないためログ出力（stderr）だけではエラーに気づけない。

通知の実現方法:

1. **osascript の `display notification`** — 外部依存なし、実装が簡単
2. **terminal-notifier** — 画像添付等が可能だが外部依存
3. **User Notifications framework（ネイティブ）** — アプリバンドル化が必要

通知対象の範囲:

- 当初は `warn` + `error` の両方を通知する方針だったが、warn は頻度が高くノイズになり得る

## Decision

`osascript` の `display notification` を使い、`error` レベルのメッセージのみを macOS 通知として表示する。

- `logger.rs` のフォーマッタ内で error レベルを検出し、同期的に osascript を実行
- 同期実行とする（非同期だとプロセス終了が先に走り通知が送信されない問題があったため）

## Consequences

- ホットキー経由の実行でもエラーに即座に気づける
- 通知の送信に osascript プロセス起動のオーバーヘッドがある（error 時のみなので許容範囲）
- 通知にアートワーク等の画像は添付できない（→ issue #10 で別途検討）
- warn レベルは通知されないため、軽微な警告はログ確認が必要
