// djay-audio-loader-helper
// グローバルホットキーを監視し、押下時に drag-into-djay を呼び出すデーモン

mod config;
mod hotkey;

use clap::Parser;
use config::{Config, NdpConfig};

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

    /// ndp-publish バイナリのフルパス（未指定時は NDP 機能無効）
    #[arg(long, value_name = "PATH")]
    ndp_publish: Option<String>,

    /// ndp-publish の出力先ベースディレクトリ（--ndp-publisher 指定時は必須）
    #[arg(long, value_name = "DIR")]
    ndp_out: Option<String>,

    /// ndp-publish の DJ ID（省略時は ndp-publish のデフォルト "dj-000"）
    #[arg(long, value_name = "ID")]
    ndp_dj_id: Option<String>,

    /// ndp-publish の DJ 名（テキスト or ロゴ画像パス）
    #[arg(long, value_name = "NAME_OR_PATH")]
    ndp_dj_name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    logger::init();

    let args = Cli::parse();

    let mut config = Config::load();
    if args.no_activate {
        config.no_activate = true;
    }

    // NDP Publisher 設定
    if let Some(ref publisher_binary) = args.ndp_publish {
        let out_dir = args
            .ndp_out
            .clone()
            .ok_or_else(|| anyhow::anyhow!("--ndp-publish を指定した場合、--ndp-out も必須です"))?;
        config.ndp = Some(NdpConfig {
            publisher_binary: publisher_binary.clone(),
            out_dir,
            dj_id: args.ndp_dj_id.clone(),
            dj_name: args.ndp_dj_name.clone(),
        });
        log::info!(
            "NDP Publisher 有効: binary={}, out={}, id={:?}, dj_name={:?}",
            publisher_binary,
            config.ndp.as_ref().unwrap().out_dir,
            config.ndp.as_ref().unwrap().dj_id,
            config.ndp.as_ref().unwrap().dj_name,
        );
    } else {
        log::info!("NDP Publisher 無効（--ndp-publish 未指定）");
    }

    // NDP 出力先のクリンナップ（起動時に既存の now_playing.json / artwork.* / .ready を削除）
    if let Some(ref ndp) = config.ndp {
        cleanup_ndp_output(ndp);
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
        "起動: deck1={:?}, deck2={:?}, ndp_publish={:?}, no_activate={}",
        config.hotkey_deck1,
        config.hotkey_deck2,
        config.hotkey_ndp_publish,
        config.no_activate
    );
    log::info!("ホットキー監視を開始します。Ctrl+C で終了。");

    hotkey::run_event_loop(&config)?;

    Ok(())
}

/// NDP 出力先ディレクトリのクリンナップ
/// 起動時に前回のデータを削除し、viewer 側に古い情報が残らないようにする
fn cleanup_ndp_output(ndp: &NdpConfig) {
    let base = expand_tilde(&ndp.out_dir);
    let dj_id = ndp.dj_id.as_deref().unwrap_or("dj-000");
    let out_dir = base.join(dj_id);

    if !out_dir.exists() {
        log::debug!(
            "NDP クリンナップ: ディレクトリ未存在 ({})",
            out_dir.display()
        );
        return;
    }

    let targets = ["now_playing.json", ".ready"];
    for name in &targets {
        let path = out_dir.join(name);
        if path.exists() {
            match std::fs::remove_file(&path) {
                Ok(_) => log::info!("NDP クリンナップ: 削除 {}", path.display()),
                Err(e) => log::warn!("NDP クリンナップ: 削除失敗 {} ({})", path.display(), e),
            }
        }
    }

    // artwork.* を削除
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("artwork.") {
                match std::fs::remove_file(entry.path()) {
                    Ok(_) => log::info!("NDP クリンナップ: 削除 {}", entry.path().display()),
                    Err(e) => log::warn!(
                        "NDP クリンナップ: 削除失敗 {} ({})",
                        entry.path().display(),
                        e
                    ),
                }
            }
        }
    }
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
