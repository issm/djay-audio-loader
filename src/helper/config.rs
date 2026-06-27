// config.rs
// ホットキー設定

/// キー修飾フラグ（CGEventFlags のサブセット）
#[derive(Debug, Clone)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub option: bool,
    pub command: bool,
}

impl Modifiers {
    pub fn as_cg_flags(&self) -> u64 {
        let mut flags: u64 = 0;
        if self.shift {
            flags |= 0x0002_0000;
        } // kCGEventFlagMaskShift
        if self.control {
            flags |= 0x0004_0000;
        } // kCGEventFlagMaskControl
        if self.option {
            flags |= 0x0008_0000;
        } // kCGEventFlagMaskAlternate
        if self.command {
            flags |= 0x0010_0000;
        } // kCGEventFlagMaskCommand
        flags
    }
}

/// ホットキー定義（仮想キーコード + 修飾キー）
#[derive(Debug, Clone)]
pub struct HotkeyDef {
    /// macOS 仮想キーコード (kVK_*)
    pub keycode: u16,
    pub modifiers: Modifiers,
    /// ロード先デッキ番号
    pub deck: u8,
}

/// アプリ設定
#[derive(Debug)]
pub struct Config {
    pub hotkey_deck1: HotkeyDef,
    pub hotkey_deck2: HotkeyDef,
    /// drag-into-djay バイナリのパス（None = 同ディレクトリから自動解決）
    pub drag_binary: Option<String>,
    /// --drop-delay に渡す値 [ms]
    pub drop_delay: u64,
    /// --no-activate を drag-into-djay に渡すか
    pub no_activate: bool,
    /// セッションログファイルのパス（None = ログ機能無効）
    pub session_file: Option<String>,
}

impl Config {
    /// 現状はハードコード。後で設定ファイル読み込みに差し替える。
    ///
    /// デフォルトホットキー:
    ///   デッキ1: Ctrl+Shift+1  (keycode 18 = kVK_ANSI_1)
    ///   デッキ2: Ctrl+Shift+2  (keycode 19 = kVK_ANSI_2)
    pub fn load() -> Self {
        let mods = Modifiers {
            shift: true,
            control: true,
            option: false,
            command: false,
        };
        Config {
            hotkey_deck1: HotkeyDef {
                keycode: 18, // kVK_ANSI_1
                modifiers: mods.clone(),
                deck: 1,
            },
            hotkey_deck2: HotkeyDef {
                keycode: 29, // kVK_ANSI_0
                modifiers: mods,
                deck: 2,
            },
            drag_binary: None,
            drop_delay: 250,
            no_activate: true,
            session_file: None,
        }
    }
}
