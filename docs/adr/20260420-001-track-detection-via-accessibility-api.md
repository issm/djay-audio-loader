# Accessibility API による選択トラック検出

## Status

Accepted

## Context

djay Pro に楽曲をロードするには、Swinsian または Music.app で選択中のトラック情報（タイトル、アーティスト、ファイルパス、行のスクリーン座標）を取得する必要がある。

取得方法の候補:

1. **AppleScript のみ** — メタ情報は取れるが、UI 要素の座標取得が困難
2. **Accessibility API (AXUIElement)** — UI 要素の属性（AXPosition, AXSize, AXSelectedRows 等）を直接取得可能
3. **Scripting Bridge** — Objective-C/Swift ブリッジだが Rust からの利用が煩雑

## Decision

Accessibility API を採用する。Rust から Core Foundation / AXUIElement の FFI を直接呼び出し、選択行の座標・メタ情報を取得する。

Music.app についてはメタ情報（タイトル、アーティスト、ファイルパス等）の取得に AppleScript を併用する（AX API ではセル内容の取得が不安定なため）。

## Consequences

- macOS のアクセシビリティ権限が必須となる（システム設定で明示的に許可が必要）
- Swinsian の UI 構造（カラム順序等）に依存するため、Swinsian のアップデートで壊れる可能性がある
- FFI コードが unsafe になるが、座標・メタ情報の両方を一度に取得でき、外部依存が最小限で済む
