# ADR: now-dj-playing publisher (ndp-publish) 連携

- **日付**: 2026-06-28
- **ステータス**: 採用

## コンテキスト

djay-audio-loader で楽曲をデッキにロードした際の楽曲情報を、now-dj-playing の viewer アプリにリアルタイムで共有したい。now-dj-playing プロジェクトには `ndp-publish` という CLI ツールがあり、楽曲ファイルパスを渡すとタグ・アートワークを抽出し共有ディレクトリに出力する。

## 決定

### 1. helper に NDP 関連オプションを追加

- `--ndp-publish <PATH>`: ndp-publish バイナリのフルパス（未指定時は NDP 機能無効）
- `--ndp-out <DIR>`: 出力先ベースディレクトリ（必須）
- `--ndp-dj-id <ID>`: DJ ID（省略時は ndp-publish デフォルト "dj-000"）
- `--ndp-dj-name <NAME>`: DJ 名またはロゴ画像パス

### 2. drag-into-djay が stdout に track_info JSON を出力

drag-into-djay 実行成功時、TrackInfo を JSON 形式で stdout に出力する。helper はこの出力をパースして最新の track_info を保持する。ログ出力は従来通り stderr のみ。

### 3. Ctrl+Shift+5 で ndp-publish を実行

helper が保持する最新の track_info の `file_path` を使い、ndp-publish を呼び出す。「ロードした曲」ではなく「再生中の曲」を任意のタイミングで制御できる設計。

### 4. 起動時クリンナップ

helper 起動時に `{ndp-out}/{dj-id}/` 内の `now_playing.json`, `.ready`, `artwork.*` を削除し、前回セッションの古い情報が viewer に残らないようにする。

## 帰結

- NDP 機能はオプトイン。`--ndp-publish` 未指定時は一切の NDP 処理を行わない
- TrackInfo に serde Serialize/Deserialize を追加（CGPoint/CGSize はカスタムシリアライザで対応）
- Ctrl+Shift+5 は track_info 未保持の状態で押しても何もしない（ログ警告のみ）
- ndp-publish の `--out`, `--id`, `--dj-name` オプションにそのまま値を渡す設計で、将来の ndp-publish 側の拡張にも追従しやすい
