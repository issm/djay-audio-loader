# ドラッグ開始位置を AXScrollArea の可視領域基準に変更

## Status

Accepted

## Context

Swinsian の曲リストペインで水平スクロール値が大きい場合、ドラッグ開始位置が可視領域外に落ち、djay Pro へのロードに失敗していた。

従来の計算:
```rust
let src = CGPoint::new(
    track.position.x + 100.0,  // 行の AXPosition.x + 固定オフセット
    track.position.y + track.size.height / 2.0,
);
```

問題: AXTable の `AXPosition` はスクロールに追従して変動する（水平スクロール時に負値になることもある）。行の `AXPosition.x` も同様に、可視領域外の座標を返す。

調査結果:
- AXTable（`pos=-161,540 size=4522x690`）— スクロールに追従、全カラム合計幅を持つ
- AXScrollArea（`pos=261,729 size=1380x481`）— スクロールに関わらず固定、可視領域を表す

## Decision

ドラッグ開始の x 座標を、AXTable を包含する AXScrollArea の `AXPosition.x` + 固定オフセット（200px）とする。y 座標は従来通り行の中央。

- `track.rs`: `find_all` で AXScrollArea を探索し、AXTable を子に持つものの位置・サイズを `TrackInfo` に保持
- `drag.rs`: `track.table_position.x + 200.0` をドラッグ開始 x に使用

## Consequences

- 水平スクロール量に関わらず、常に可視領域内からドラッグが開始される
- AXScrollArea の走査コストが追加されるが、既存の `find_all` と同様の深さ（8）で実用上問題ない
- オフセット 200px は固定値のため、極端に幅が狭いウィンドウでは調整が必要になる可能性がある
