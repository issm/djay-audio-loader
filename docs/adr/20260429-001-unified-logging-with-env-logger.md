# ログ出力機構の統一（env_logger）

## Status

Accepted

## Context

初期実装では各モジュールで個別に `println!` / `eprintln!` を使ったログ出力が散在していた。ログレベル制御、タイムスタンプ付与、出力フォーマットの統一ができておらず、デバッグ時に不便だった。

候補:

1. **env_logger + log クレート** — Rust エコシステムで最も広く使われる組み合わせ。軽量で設定が簡単
2. **tracing** — 構造化ログ・非同期対応が強みだが、本プロジェクトの規模にはオーバースペック
3. **自前実装** — 柔軟だが車輪の再発明

## Decision

`log` クレート（ファサード）+ `env_logger`（実装）を採用し、`src/logger.rs` に初期化処理を集約する。

- デフォルトログレベル: `info`
- `RUST_LOG` 環境変数でレベル制御可能
- タイムスタンプ: ローカルタイム（JST）付き、`chrono` で生成
- フォーマット: `{timestamp} [{level}] {message}`
- `error` レベルのメッセージは macOS 通知（osascript）も同時に表示

## Consequences

- 全モジュールから `log::info!()`, `log::debug!()` 等のマクロで統一的にログ出力できる
- `RUST_LOG=debug` でデバッグログを有効化でき、トラブルシュートが容易になった
- `chrono` クレートへの依存が追加された
- error 時の macOS 通知により、ヘルパ経由の実行でもエラーに気づきやすい
