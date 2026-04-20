// djay-audio-loader-helper
// グローバルホットキーを監視し、押下時に drag-into-djay を呼び出すデーモン

mod config;
mod hotkey;

use clap::Parser;
use config::Config;

#[derive(Parser, Debug)]
#[command(
    name = "djay-audio-loader-helper",
    about = "グローバルホットキーを監視し drag-into-djay を呼び出すヘルパ"
)]
struct Cli {
    /// アプリのアクティブ化をスキップする（drag-into-djay に --no-activate を渡す）
    #[arg(long, default_value_t = false)]
    no_activate: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let mut config = Config::load();
    if args.no_activate {
        config.no_activate = true;
    }

    eprintln!(
        "[helper] 起動: deck1={:?}, deck2={:?}, no_activate={}",
        config.hotkey_deck1, config.hotkey_deck2, config.no_activate
    );
    eprintln!("[helper] ホットキー監視を開始します。Ctrl+C で終了。");

    hotkey::run_event_loop(&config)?;

    Ok(())
}
