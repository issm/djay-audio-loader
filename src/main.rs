mod cli;
mod drag;
mod logger;
mod notify;
mod session_log;
mod track;

use anyhow::Result;

fn run() -> Result<()> {
    let args = cli::parse();

    if args.deck < 1 || args.deck > 2 {
        anyhow::bail!("デッキ番号は 1 または 2 を指定してください");
    }

    log::info!("CLI パース完了: deck={}", args.deck);

    let track = track::get_selected_track()?;
    let comment_info = if track.comment.is_empty() {
        String::new()
    } else {
        format!(" (コメント: {})", track.comment)
    };
    log::info!(
        "トラック取得完了: [{}] {} / {}{}",
        track.source,
        track.title,
        track.file_path,
        comment_info
    );

    // ファイルパスが取得できている場合、実際に存在するか確認する
    if !track.file_path.is_empty() {
        if std::path::Path::new(&track.file_path).exists() {
            log::info!("ファイルパスチェック: OK ({})", track.file_path);
        } else {
            anyhow::bail!("ファイルが見つかりません: {}", track.file_path);
        }
    } else {
        log::info!("ファイルパスチェック: スキップ（パス未取得）");
    }

    drag::drag_to_djay(&track, args.deck, args.drop_delay, args.no_activate)?;
    log::info!("ドラッグ完了: デッキ {} にロードしました", args.deck);

    // セッションログ追記
    if let Some(ref session_file) = args.session_file {
        if let Err(e) = session_log::append_track(session_file, &track, args.deck) {
            log::warn!("セッションログの追記に失敗しました: {}", e);
        }
    }

    // 成功通知
    if let Err(e) = notify::notify_success(&track, args.deck) {
        log::warn!("通知の送信に失敗しました: {}", e);
    }

    // track_info を JSON で stdout に出力（helper が受け取る）
    if let Ok(json) = serde_json::to_string(&track) {
        println!("{}", json);
    }

    Ok(())
}

fn main() {
    logger::init();

    if let Err(e) = run() {
        log::error!("{}", e);

        // エラー通知
        if let Err(notify_err) = notify::notify_error(&e.to_string()) {
            log::warn!("エラー通知の送信に失敗しました: {}", notify_err);
        }

        std::process::exit(1);
    }
}
