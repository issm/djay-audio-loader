// logger.rs
// ログ出力の初期化・設定を集約するモジュール
//
// 使い方:
//   logger::init();
//
// ログレベルは RUST_LOG 環境変数で制御できる（デフォルト: info）
//   RUST_LOG=debug ./drag-into-djay -d 1
//
// タイムスタンプはシステムのローカルタイム（日本時間）で出力される
//
// error レベルのメッセージは macOS 通知としても表示される

use std::io::Write;

/// error レベルのメッセージを macOS 通知で表示する（同期）
fn notify(message: &str) {
    let script = format!(
        "display notification \"{}\" with title \"djay-audio-loader: エラー\"",
        message.replace('"', "\\\""),
    );
    let _ = std::process::Command::new("osascript")
        .args(["-e", &script])
        .output();
}

pub fn init() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");

            if matches!(record.level(), log::Level::Error) {
                notify(&record.args().to_string());
            }

            writeln!(buf, "{} [{}] {}", ts, record.level(), record.args())
        })
        .init();
}
