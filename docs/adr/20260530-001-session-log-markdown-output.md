# 選曲ログを Markdown ファイルに書き出す

## Status

Accepted

## Context

djay Pro へのトラックロードはホットキー経由で行うため、操作ログがターミナルに埋もれてしまい、セッション後に選曲を振り返る手段がなかった。

要件:
- 1 DJ セッション（= helper の1回の起動〜終了）を1つの Markdown ファイルで管理
- Obsidian で即座に参照・編集できること
- TaskNotes / OKF (Open Knowledge Format) 互換のフロントマターを持つこと
- トラック情報（連番・時刻・経過時間・デッキ番号・曲名・アーティスト・アートワーク）を追記
- helper が Ctrl+C で kill されてもデータが失われないこと

## Decision

### アーキテクチャ

helper → drag-into-djay の2階層で、セッションファイルパスを CLI オプション経由で伝達する。

1. **helper 起動時**: セッションディレクトリ・md ファイル・artworks/ ディレクトリを生成
2. **ホットキー押下時**: `drag-into-djay --session-file <path>` で子プロセスにパスを渡す
3. **ロード成功後**: `session_log::append_track()` で md ファイルに1行追記＋アートワーク抽出・保存

### CLI オプション

helper 側に2種類のオプションを用意:
- `--session-basedir (-D)`: ベースディレクトリを指定。配下にタイムスタンプ名のサブディレクトリを自動生成
- `--session-dir (-S)`: セッションディレクトリを直接指定（既存なら追記）

drag-into-djay 側:
- `--session-file`: helper から渡されるセッション md ファイルの絶対パス

いずれも未指定時はセッションログ機能を無効化（既存動作に影響しない）。

### ディレクトリ構造

```
<basedir>/
└── 2026-06-26-2315/
    ├── 2026-06-26-2315.md
    └── artworks/
        ├── 001.jpg
        ├── 002.jpg
        └── ...
```

### ファイル形式

フロントマター（OKF / TaskNotes 互換）:
```yaml
---
type: DJ Session
title: "2026-06-26-2315"
description: DJ セッションログ
date: 2026-06-26
tags: [dj, session]
timestamp: 2026-06-26T23:15:00+09:00
---
```

本文（テンプレートベース、1行1トラック）:
```
- 001. **23:15:30** (+0:00:00) Deck 1｜Track Name / Artist Name
```

### アートワーク抽出

- MP3: `id3` クレートで APIC フレームから取得
- M4A: `mp4ameta` クレートで取得
- 640px 正方形に収まるよう `image` クレートでリサイズ
- `artworks/{no:03}.{ext}` として保存

### ファイル書き込み戦略

追記のたびに open → write → close する。ファイルを握り続けないことで:
- Ctrl+C による中断に耐える
- Obsidian 側でリアルタイムに変更を検知できる

### 実装箇所

- `src/session_log.rs`: 追記ロジック・アートワーク抽出・テンプレート展開
- `src/helper/main.rs`: セッションディレクトリ初期化、フロントマター生成
- `src/cli.rs`: `--session-file` オプション追加
- `src/main.rs`: ロード成功後に `session_log::append_track()` 呼び出し

## Consequences

- セッション単位でセットリストが Markdown として残り、Obsidian で即座に確認・編集できる
- OKF / TaskNotes 互換のため、TaskForge 等のツールからも参照可能
- アートワーク抽出により視覚的にトラックを識別できるが、画像容量が蓄積する（640px リサイズで緩和）
- テンプレートは現状ハードコードだが、将来的に設定ファイルで変更可能にする余地がある
- `--session-basedir` / `--session-dir` の2系統は用途によって使い分け可能だが、オプションが増える代わりに柔軟性を確保
