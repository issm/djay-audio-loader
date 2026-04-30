mod cli;
mod drag;
mod logger;
mod track;

use anyhow::Result;

fn run() -> Result<()> {
    let args = cli::parse();

    if args.deck < 1 || args.deck > 2 {
        anyhow::bail!("デッキ番号は 1 または 2 を指定してください");
    }

    log::info!("CLI パース完了: deck={}", args.deck);

    let track = track::get_selected_track()?;
    log::info!(
        "トラック取得完了: [{}] {} / {}",
        track.source,
        track.title,
        track.file_path
    );

    drag::drag_to_djay(&track, args.deck, args.drop_delay, args.no_activate)?;
    log::info!("ドラッグ完了: デッキ {} にロードしました", args.deck);

    Ok(())
}

fn main() {
    logger::init();

    if let Err(e) = run() {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
