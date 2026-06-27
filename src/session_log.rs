// session_log.rs
// セッションログファイルへのトラック情報追記

use crate::track::TrackInfo;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// 行テンプレートで使用可能な変数
///
/// - `{no}`: 連番
/// - `{time}`: ロード時刻 (HH:MM:SS)
/// - `{elapsed}`: 1曲目からの経過時間 (H:MM:SS)
/// - `{deck}`: デッキ番号
/// - `{title}`: 曲名
/// - `{artist}`: アーティスト名
/// - `{artwork}`: アートワーク相対パス
const DEFAULT_LINE_TEMPLATE: &str =
    "{no}. **{time}** (+{elapsed}) Deck {deck} | {title} / {artist}";

/// テンプレートに変数を適用して1行を生成する
fn render_line(
    template: &str,
    no: u32,
    time: &str,
    elapsed: &str,
    deck: u8,
    title: &str,
    artist: &str,
    artwork: &str,
) -> String {
    template
        .replace("{no}", &no.to_string())
        .replace("{time}", time)
        .replace("{elapsed}", elapsed)
        .replace("{deck}", &deck.to_string())
        .replace("{title}", title)
        .replace("{artist}", artist)
        .replace("{artwork}", artwork)
}

/// セッションファイルから現在の行数（= 次の連番）を取得する
fn count_entries(session_file: &Path) -> u32 {
    let content = std::fs::read_to_string(session_file).unwrap_or_default();
    // フロントマター以降の行で、番号付きリスト項目をカウント
    let mut in_body = false;
    let mut frontmatter_delimiters = 0;
    let mut count = 0u32;
    for line in content.lines() {
        if line.trim() == "---" {
            frontmatter_delimiters += 1;
            if frontmatter_delimiters == 2 {
                in_body = true;
            }
            continue;
        }
        if in_body && !line.trim().is_empty() {
            count += 1;
        }
    }
    count
}

/// 1曲目の時刻を取得する（経過時間の計算に使用）
fn first_entry_time(session_file: &Path) -> Option<chrono::NaiveTime> {
    let content = std::fs::read_to_string(session_file).ok()?;
    let mut in_body = false;
    let mut frontmatter_delimiters = 0;
    for line in content.lines() {
        if line.trim() == "---" {
            frontmatter_delimiters += 1;
            if frontmatter_delimiters == 2 {
                in_body = true;
            }
            continue;
        }
        if in_body && !line.trim().is_empty() {
            // "1. **HH:MM:SS** ..." から時刻を抽出
            if let Some(start) = line.find("**") {
                if let Some(end) = line[start + 2..].find("**") {
                    let time_str = &line[start + 2..start + 2 + end];
                    return chrono::NaiveTime::parse_from_str(time_str, "%H:%M:%S").ok();
                }
            }
            return None;
        }
    }
    None
}

/// 経過時間をフォーマットする (H:MM:SS)
fn format_elapsed(elapsed_secs: i64) -> String {
    let h = elapsed_secs / 3600;
    let m = (elapsed_secs % 3600) / 60;
    let s = elapsed_secs % 60;
    format!("{}:{:02}:{:02}", h, m, s)
}

/// セッションファイルにトラック情報を1行追記する
pub fn append_track(session_file: &str, track: &TrackInfo, deck: u8) -> Result<()> {
    let path = Path::new(session_file);
    let now = chrono::Local::now();
    let time_str = now.format("%H:%M:%S").to_string();

    // 連番
    let no = count_entries(path) + 1;

    // 経過時間
    let elapsed = if no == 1 {
        "0:00:00".to_string()
    } else {
        let current_time = now.time();
        match first_entry_time(path) {
            Some(first_time) => {
                let diff = current_time.signed_duration_since(first_time);
                format_elapsed(diff.num_seconds().max(0))
            }
            None => "0:00:00".to_string(),
        }
    };

    // TODO: アートワーク抽出は後続で実装
    let artwork = String::new();

    let line = render_line(
        DEFAULT_LINE_TEMPLATE,
        no,
        &time_str,
        &elapsed,
        deck,
        &track.title,
        &track.artist,
        &artwork,
    );

    // open → write → close
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", line)?;

    log::info!("セッションログ追記: {}", line);
    Ok(())
}
