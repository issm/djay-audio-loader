#!/bin/bash
# アートワーク抽出の検証スクリプト

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TEMP_DIR="/tmp/djay-artwork-test"

mkdir -p "$TEMP_DIR/src"

cat > "$TEMP_DIR/Cargo.toml" << 'EOF'
[package]
name = "artwork-test"
version = "0.1.0"
edition = "2021"

[dependencies]
id3 = "1.13"
mp4ameta = "0.11"
anyhow = "1.0"
EOF

cat > "$TEMP_DIR/src/main.rs" << 'EOF'
use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;

fn extract_artwork(file_path: &str) -> Result<Option<Vec<u8>>> {
    let path = Path::new(file_path);
    let ext = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "mp3" => {
            let tag = id3::Tag::read_from_path(path)?;
            if let Some(picture) = tag.pictures().next() {
                println!("✅ MP3: アートワーク検出");
                println!("   形式: {:?}", picture.picture_type);
                println!("   MIME: {}", picture.mime_type);
                println!("   サイズ: {} bytes", picture.data.len());
                return Ok(Some(picture.data.to_vec()));
            }
        }
        "m4a" | "mp4" | "m4p" => {
            let tag = mp4ameta::Tag::read_from_path(path)?;
            if let Some(artwork) = tag.artwork() {
                println!("✅ M4A: アートワーク検出");
                println!("   形式: {:?}", artwork.fmt);
                println!("   サイズ: {} bytes", artwork.data.len());
                let data = artwork.data.to_vec();
                return Ok(Some(data));
            }
        }
        _ => {
            println!("⚠️  未対応の形式: {}", ext);
        }
    }

    println!("❌ アートワークなし");
    Ok(None)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使い方: {} <audio_file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("ファイル: {}", file_path);

    if let Some(data) = extract_artwork(file_path)? {
        let output_path = "/tmp/extracted_artwork.jpg";
        fs::write(output_path, data)?;
        println!("\n💾 保存先: {}", output_path);
        println!("   確認: open {}", output_path);
    }

    Ok(())
}
EOF

echo "📦 ビルド中..."
cd "$TEMP_DIR"
cargo build --release 2>&1 | tail -5

echo ""
echo "✅ ビルド完了"
echo ""
echo "使い方:"
echo "  $TEMP_DIR/target/release/artwork-test <audio_file_path>"
echo ""
echo "例:"
echo "  $TEMP_DIR/target/release/artwork-test '/path/to/music.m4a'"
