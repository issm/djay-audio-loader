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
// warn / error レベルのメッセージは macOS 通知としても表示される

use std::io::Write;

/// warn / error レベルのメッセージを macOS 通知で表示する（非同期）
fn notify(level: log::Level, message: &str) {
    let title = match level {
        log::Level::Error => "djay-audio-loader: エラー",
        log::Level::Warn => "djay-audio-loader: 警告",
        _ => return,
    };
    let title = title.to_string();
    let message = message.to_string();

    std::thread::spawn(move || {
        let script = format!(
            "display notification \"{}\" with title \"{}\"",
            message.replace('"', "\\\""),
            title.replace('"', "\\\""),
        );
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output();
    });
}

pub fn init() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");

            if matches!(record.level(), log::Level::Warn | log::Level::Error) {
                notify(record.level(), &record.args().to_string());
            }

            writeln!(buf, "{} [{}] {}", ts, record.level(), record.args())
        })
        .init();
}
