mod cli;
mod drag;
mod track;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::parse();

    if args.deck < 1 || args.deck > 2 {
        anyhow::bail!("デッキ番号は 1 または 2 を指定してください");
    }

    eprintln!("[1] CLI パース完了: deck={}", args.deck);

    let track = track::get_selected_track()?;
    eprintln!(
        "[2] トラック取得完了: [{}] {} / {}",
        track.source, track.title, track.file_path
    );

    drag::drag_to_djay(&track, args.deck, args.drop_delay)?;
    eprintln!("[3] ドラッグ完了: デッキ {} にロードしました", args.deck);

    Ok(())
}
