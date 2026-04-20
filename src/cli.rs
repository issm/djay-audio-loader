use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "drag-into-djay",
    about = "選択中の楽曲を djay Pro の指定デッキにロードする"
)]
pub struct Cli {
    /// ロード先のデッキ番号 (1 or 2)
    #[arg(short, long, value_name = "DECK_NO")]
    pub deck: u8,

    /// ドロップ前のホバー待機時間 [ms]
    #[arg(long, value_name = "MS", default_value_t = 250)]
    pub drop_delay: u64,

    /// アプリのアクティブ化をスキップする（helper から呼ぶ場合など）
    #[arg(long, default_value_t = false)]
    pub no_activate: bool,
}

pub fn parse() -> Cli {
    Cli::parse()
}
