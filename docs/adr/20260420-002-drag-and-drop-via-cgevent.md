# CGEvent によるドラッグ&ドロップ

## Status

Accepted

## Context

Swinsian / Music.app の選択行を djay Pro の波形エリアにロードする方法が必要。djay Pro は通常のファイルドロップを受け付けるが、プログラムからドロップを実行する方法として以下の候補がある:

1. **CGEvent によるマウス操作シミュレーション** — MouseDown → Drag → MouseUp をプログラムで生成
2. **NSPasteboard + ドラッグセッション** — AppKit のドラッグ API を利用（アプリケーションバンドル化が必要）
3. **AppleScript の drag 操作** — 標準では存在しない

## Decision

CGEvent を使ったマウス操作シミュレーションを採用する。

- ソースアプリの選択行上で MouseDown
- djay Pro の波形エリアまで段階的に Drag イベントを送信（10ステップ）
- ドロップ先でホバー待機後に MouseUp

djay Pro の波形エリア座標は Accessibility API でウィンドウ位置を取得し、事前検証で確定した固定オフセットを加算して算出する。

## Consequences

- CLI ツールのまま動作し、アプリケーションバンドル化が不要
- djay Pro のウィンドウレイアウトが変わると固定オフセットの再調整が必要
- ドラッグ中にユーザーがマウスを動かすと干渉する可能性がある
- アクセシビリティ権限に加え、入力監視の権限も必要（CGEventPost のため）
