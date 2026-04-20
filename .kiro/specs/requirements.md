# 要件

## 概要

Swinsian で選択中の楽曲を djay Pro の特定デッキにロードするツール群。

---

## ツール構成

### 1. `drag-into-djay` — ドラッグ&ドロップ実行ツール

#### コマンドライン仕様

```
drag-into-djay -f <audio_file> -d <deck_no>
```

#### オプション

- `-f, --file <audio_file>`: ロード対象の楽曲ファイルのフルパス
- `-d, --deck <deck_no>`: ロード先の djay Pro におけるデッキ番号

#### 動作

1. Accessibility API でドラッグ元アプリ（現在は Swinsian、将来的に Music.app 等にも対応予定）の選択中アイテムの UI 上の座標を動的取得
2. djay Pro の指定デッキの波形エリア座標を Accessibility API で動的取得
3. Swinsian の選択アイテムから djay Pro の波形エリアへ CGEvent でドラッグ&ドロップを実行

#### 前提条件

- Swinsian・djay Pro の両方が起動済みかつフォアグラウンド表示されていること（将来的に Music.app 等にも対応予定）
- Accessibility の権限が付与されていること

---

### 2. `djay-audio-loader-helper` — ホットキー監視デーモン

#### 動作

- `NSWorkspace` で djay Pro の起動を監視し、起動したらホットキー監視を有効化
- djay Pro が終了したらホットキー監視を無効化
- `launchd` エージェントとしてログイン時に常駐起動

#### ホットキー

- 未定（設定ファイルで指定可能とする）

#### イベント処理

- `CGEventTap`（active モード）でグローバルホットキーを監視
- ホットキー押下時:
  - `djay-audio-loader-helper` が `drag-into-djay` を呼び出す
  - イベントを消費し、アクティブなアプリへは伝達しない
- ホットキー以外のイベントは通常通り伝達

#### 権限

- Accessibility の権限が必要
