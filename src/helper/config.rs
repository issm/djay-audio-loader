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

/// NDP (Now DJ Playing) Publisher 設定
#[derive(Debug, Clone)]
pub struct NdpConfig {
    /// ndp-publish バイナリのフルパス
    pub publisher_binary: String,
    /// 出力先ベースディレクトリ (--out)
    pub out_dir: String,
    /// DJ ID (--id)、未指定時は ndp-publish のデフォルト ("dj-000")
    pub dj_id: Option<String>,
    /// DJ 名 (--dj-name)
    pub dj_name: Option<String>,
}

/// アプリ設定
#[derive(Debug)]
pub struct Config {
    pub hotkey_deck1: HotkeyDef,
    pub hotkey_deck2: HotkeyDef,
    /// Ctrl+Shift+5: ndp-publish 実行ホットキー
    pub hotkey_ndp_publish: HotkeyDef,
    /// drag-into-djay バイナリのパス（None = 同ディレクトリから自動解決）
    pub drag_binary: Option<String>,
    /// --drop-delay に渡す値 [ms]
    pub drop_delay: u64,
    /// --no-activate を drag-into-djay に渡すか
    pub no_activate: bool,
    /// セッションログファイルのパス（None = ログ機能無効）
    pub session_file: Option<String>,
    /// NDP Publisher 設定（None = NDP 機能無効）
    pub ndp: Option<NdpConfig>,
}

impl Config {
    /// 現状はハードコード。後で設定ファイル読み込みに差し替える。
    ///
    /// デフォルトホットキー:
    ///   デッキ1: Ctrl+Shift+1  (keycode 18 = kVK_ANSI_1)
    ///   デッキ2: Ctrl+Shift+0  (keycode 29 = kVK_ANSI_0)
    ///   NDP publish: Ctrl+Shift+5  (keycode 23 = kVK_ANSI_5)
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
                modifiers: mods.clone(),
                deck: 2,
            },
            hotkey_ndp_publish: HotkeyDef {
                keycode: 23, // kVK_ANSI_5
                modifiers: mods,
                deck: 0, // NDP publish 用なので deck は使わない
            },
            drag_binary: None,
            drop_delay: 250,
            no_activate: true,
            session_file: None,
            ndp: None,
        }
    }
}
