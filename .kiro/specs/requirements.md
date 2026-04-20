# 要件

## 概要

Swinsian または Music.app で選択中の楽曲を djay Pro の特定デッキにロードするツール群。

---

## 既知の課題

### iTunes ケースで一部の曲がドロップできない

Music.app からのドラッグ時、特定の曲で djay Pro がドロップを受け付けない現象が確認されている。

- 再現曲: La Lu Sugarin' (こんぺとリリィ)、感情グラス (上伊那ぼたん)、Ciao Ciao (Sophia la Mode)、あいでんてぃんてぃん (feat. 初音ミク, 可不 & ずんだもん) (榊みむ)
- Swinsian では同じ曲でも正常にロードできる
- ログ上はドラッグ操作が正常に完了しており、コード側での区別は不可能
- Music.app がドラッグセッションにファイルパス情報を乗せない曲が存在する可能性がある
- 原因不明のため保留中


### 1. `drag-into-djay` — ドラッグ&ドロップ実行ツール

#### コマンドライン仕様

```
drag-into-djay -d <deck_no>
```

#### オプション

- `-d, --deck <deck_no>`: ロード先の djay Pro におけるデッキ番号

#### 動作

1. Accessibility API でドラッグ元アプリの選択中アイテムの UI 上の座標を動的取得
   - ドラッグ元アプリの選択は以下の優先順位で決定する:
     1. Swinsian・Music.app のうちアクティブ（フォアグラウンド）なものを優先
     2. 両方非アクティブの場合: Swinsian → Music.app の順
2. djay Pro の指定デッキの波形エリア座標を Accessibility API で動的取得
3. 選択アイテムから djay Pro の波形エリアへ CGEvent でドラッグ&ドロップを実行

#### 前提条件

- Swinsian または Music.app、および djay Pro が起動済みかつフォアグラウンド表示されていること
- Accessibility の権限が付与されていること

---

### 2. `djay-audio-loader-helper` — ホットキー監視デーモン

#### 動作

- `NSWorkspace` で djay Pro の起動を監視し、起動したらホットキー監視を有効化
- djay Pro が終了したらホットキー監視を無効化
- `launchd` エージェントとしてログイン時に常駐起動

#### ホットキー

- デッキ1: `Ctrl+Shift+1`
- デッキ2: `Ctrl+Shift+0`
- TODO: 設定ファイルで変更可能とする

#### イベント処理

- `CGEventTap`（active モード）でグローバルホットキーを監視
- ホットキー押下時:
  - `djay-audio-loader-helper` が `drag-into-djay` を呼び出す
  - イベントを消費し、アクティブなアプリへは伝達しない
- ホットキー以外のイベントは通常通り伝達

#### 権限

- Accessibility の権限が必要
