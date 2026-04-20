// djay-audio-loader-helper
// グローバルホットキーを監視し、押下時に drag-into-djay を呼び出すデーモン

mod config;
mod hotkey;

use anyhow::Result;
use config::Config;

fn main() -> Result<()> {
    let config = Config::load();
    eprintln!(
        "[helper] 起動: deck1={:?}, deck2={:?}",
        config.hotkey_deck1, config.hotkey_deck2
    );
    eprintln!("[helper] ホットキー監視を開始します。Ctrl+C で終了。");

    hotkey::run_event_loop(&config)?;

    Ok(())
}
