mod cli;
mod drag;
mod track;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::parse();

    if args.deck < 1 || args.deck > 2 {
        anyhow::bail!("デッキ番号は 1 または 2 を指定してください");
    }

    let track = track::get_selected_track()?;
    eprintln!(
        "取得: [{}] {} / {} ({})",
        track.source, track.title, track.artist, track.file_path
    );

    drag::drag_to_djay(&track, args.deck)?;
    eprintln!("完了: デッキ {} にロードしました", args.deck);

    Ok(())
}
