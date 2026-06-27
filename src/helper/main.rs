// djay-audio-loader-helper
// グローバルホットキーを監視し、押下時に drag-into-djay を呼び出すデーモン

mod config;
mod hotkey;

use clap::Parser;
use config::Config;

// logger は src/ 直下にあるため、helper バイナリからは #[path] で参照する
#[path = "../logger.rs"]
mod logger;

#[derive(Parser, Debug)]
#[command(
    name = "djay-audio-loader-helper",
    about = "グローバルホットキーを監視し drag-into-djay を呼び出すヘルパ"
)]
struct Cli {
    /// アプリのアクティブ化をスキップする（drag-into-djay に --no-activate を渡す）
    #[arg(long, default_value_t = false)]
    no_activate: bool,

    /// セッションログを配置するディレクトリ（未指定時はログ機能無効）
    #[arg(short = 'S', long)]
    session_dir: Option<String>,
}

fn main() -> anyhow::Result<()> {
    logger::init();

    let args = Cli::parse();

    let mut config = Config::load();
    if args.no_activate {
        config.no_activate = true;
    }

    // セッションディレクトリの準備
    let session_file = if let Some(ref dir) = args.session_dir {
        let session_file = setup_session_dir(dir)?;
        log::info!("セッションログ: {}", session_file.display());
        Some(session_file)
    } else {
        None
    };

    config.session_file = session_file.map(|p| p.to_string_lossy().into_owned());

    log::info!(
        "起動: deck1={:?}, deck2={:?}, no_activate={}",
        config.hotkey_deck1,
        config.hotkey_deck2,
        config.no_activate
    );
    log::info!("ホットキー監視を開始します。Ctrl+C で終了。");

    hotkey::run_event_loop(&config)?;

    Ok(())
}

/// セッションディレクトリを作成し、md ファイルパスを返す
fn setup_session_dir(base_dir: &str) -> anyhow::Result<std::path::PathBuf> {
    use std::path::PathBuf;

    // ~ 展開
    let base = if base_dir.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(base_dir.replacen('~', &home, 1))
    } else {
        PathBuf::from(base_dir)
    };

    // セッション名: yyyy-mm-dd-HHmm
    let now = chrono::Local::now();
    let session_name = now.format("%Y-%m-%d-%H%M").to_string();

    // ディレクトリ構造: <base>/<session_name>/artworks/
    let session_dir = base.join(&session_name);
    let artworks_dir = session_dir.join("artworks");
    std::fs::create_dir_all(&artworks_dir)?;

    // md ファイルパス
    let md_path = session_dir.join(format!("{}.md", session_name));

    // フロントマターを書き込み（ファイルが存在しない場合のみ）
    if !md_path.exists() {
        let frontmatter = format!(
            "---\ntype: DJ Session\ntitle: \"{}\"\ndescription: DJ セッションログ\ndate: {}\ntags: [dj, session]\ntimestamp: {}\n---\n",
            session_name,
            now.format("%Y-%m-%d"),
            now.format("%Y-%m-%dT%H:%M:%S%:z"),
        );
        std::fs::write(&md_path, frontmatter)?;
    }

    Ok(md_path)
}
