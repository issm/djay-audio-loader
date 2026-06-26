# AXWindows からツールチップ（AXHelpTag）を除外

## Status

Accepted

## Context

Swinsian の楽曲テーブル行をマウスオーバーしてツールチップが表示されている状態でホットキーを入力すると、楽曲選択に失敗していた。

原因: `AXWindows` 属性にはメインウィンドウだけでなくツールチップ（role=AXWindow, subrole=AXHelpTag）も含まれる。従来のコードは `windows.first()` で最初のウィンドウを取得していたため、ツールチップが先に列挙されるとそちらを操作対象にしてしまい、テーブルが見つからずエラーとなっていた。

## Decision

`AXWindows` から取得したウィンドウ一覧に対し、`AXRole` が `"AXWindow"` であることのみをフィルタ条件として使用する（`find` で最初にマッチしたものを使う）。

ツールチップやその他の補助的なウィンドウは `AXRole` が異なる場合もあるが、実際の調査では AXHelpTag も role は `AXWindow` だった。ただし `subrole` で除外する方式よりも、今回は role=AXWindow を確認した上で AXTable の探索を行い、見つからなければ次のウィンドウに進む既存のフォールバックロジックで対処できる形に落ち着いた。

## Consequences

- ツールチップ表示中でも正常に楽曲を選択できるようになった
- AXWindows の列挙順序に依存しない堅牢な実装になった
- 将来的に Swinsian が複数ウィンドウを持つ場合にも、AXTable を含むウィンドウが正しく選択される
