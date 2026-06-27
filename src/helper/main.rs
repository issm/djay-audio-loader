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

    /// セッションログのベースディレクトリ（この下にタイムスタンプ名のサブディレクトリを自動生成）
    #[arg(short = 'D', long)]
    session_basedir: Option<String>,

    /// セッションディレクトリを直接指定（既存なら追記、なければ新規作成）
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
        let session_file = setup_session_dir_direct(dir)?;
        log::info!("セッションログ: {}", session_file.display());
        Some(session_file)
    } else if let Some(ref basedir) = args.session_basedir {
        let session_file = setup_session_basedir(basedir)?;
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

/// ~ 展開を行う
fn expand_tilde(path: &str) -> std::path::PathBuf {
    use std::path::PathBuf;
    if path.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(path.replacen('~', &home, 1))
    } else {
        PathBuf::from(path)
    }
}

/// --session-basedir: ベースディレクトリ下にタイムスタンプ名のサブディレクトリを自動生成
fn setup_session_basedir(base_dir: &str) -> anyhow::Result<std::path::PathBuf> {
    let base = expand_tilde(base_dir);

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

/// --session-dir: セッションディレクトリを直接指定（既存なら追記、なければ新規作成）
fn setup_session_dir_direct(dir: &str) -> anyhow::Result<std::path::PathBuf> {
    let session_dir = expand_tilde(dir);
    let artworks_dir = session_dir.join("artworks");
    std::fs::create_dir_all(&artworks_dir)?;

    // ディレクトリ名をセッション名として使う
    let session_name = session_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "session".to_string());

    // md ファイルパス
    let md_path = session_dir.join(format!("{}.md", session_name));

    // フロントマターを書き込み（ファイルが存在しない場合のみ）
    if !md_path.exists() {
        let now = chrono::Local::now();
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
