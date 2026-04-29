// logger.rs
// ログ出力の初期化・設定を集約するモジュール
//
// 使い方:
//   logger::init();
//
// ログレベルは RUST_LOG 環境変数で制御できる（デフォルト: info）
//   RUST_LOG=debug ./drag-into-djay -d 1

pub fn init() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_module_path(false)
        .init();
}
