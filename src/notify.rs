// notify.rs
// macOS の通知センターに通知を表示する機能
// osascript の display notification を使用

use crate::track::TrackInfo;
use anyhow::Result;
use std::process::Command;

// ---- 通知送信 ---------------------------------------------------------------

/// 成功通知を送信
pub fn notify_success(track: &TrackInfo, deck: u8) -> Result<()> {
    let title = format!("✅ デッキ {} にロード", deck);

    let mut subtitle = track.title.clone();
    if !track.artist.is_empty() {
        subtitle = format!("{} - {}", track.artist, track.title);
    }

    let mut message = String::new();
    if !track.comment.is_empty() {
        message = format!("💬 {}", track.comment);
    }

    send_notification(&title, &subtitle, &message)?;
    Ok(())
}

/// エラー通知を送信
pub fn notify_error(error_message: &str) -> Result<()> {
    let title = "❌ エラー";
    send_notification(title, error_message, "")?;
    Ok(())
}

fn send_notification(title: &str, subtitle: &str, message: &str) -> Result<()> {
    // osascript で通知を送信
    let script = format!(
        r#"display notification "{}" with title "{}" subtitle "{}""#,
        escape_applescript(message),
        escape_applescript(title),
        escape_applescript(subtitle)
    );

    let output = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("通知の送信に失敗しました: {}", err);
    }

    Ok(())
}

fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', " ")
}
