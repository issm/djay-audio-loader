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
}

pub fn parse() -> Cli {
    Cli::parse()
}
