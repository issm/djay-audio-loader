// hotkey.rs
// CGEventTap によるグローバルホットキー監視

use anyhow::{Result, anyhow};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use crate::config::{Config, HotkeyDef};

// ---- CGEventTap FFI --------------------------------------------------------

#[allow(non_camel_case_types)]
type CFMachPortRef = *mut c_void;
#[allow(non_camel_case_types)]
type CFRunLoopSourceRef = *mut c_void;
#[allow(non_camel_case_types)]
type CFRunLoopRef = *mut c_void;
#[allow(non_camel_case_types)]
type CGEventRef = *mut c_void;
#[allow(non_camel_case_types)]
type CGEventTapProxy = *mut c_void;

// CGEventMask: 1 << CGEventType
// kCGEventKeyDown = 10
const CG_EVENT_KEY_DOWN: u32 = 10;
const CG_EVENT_MASK_KEY_DOWN: u64 = 1 << CG_EVENT_KEY_DOWN;

// kCGEventTapOptionDefault (active tap, can filter events) = 0
const CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;
// kCGHIDEventTap = 0
const CG_HID_EVENT_TAP: u32 = 0;
// kCGHeadInsertEventTap = 0
const CG_HEAD_INSERT_EVENT_TAP: u32 = 0;

// CGEventField: kCGKeyboardEventKeycode = 9
const CG_KEYBOARD_EVENT_KEYCODE: i32 = 9;

unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: unsafe extern "C" fn(
            proxy: CGEventTapProxy,
            event_type: u32,
            event: CGEventRef,
            user_info: *mut c_void,
        ) -> CGEventRef,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CFMachPortCreateRunLoopSource(
        allocator: *mut c_void,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;

    fn CFRunLoopGetCurrent() -> CFRunLoopRef;

    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: *const c_void);

    fn CFRunLoopRun();

    fn CGEventGetIntegerValueField(event: CGEventRef, field: i32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;

    // タップの有効/無効切り替え
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}

// kCFRunLoopCommonModes — CoreFoundation の定数シンボルを直接参照
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopCommonModes: *const c_void;
}

// ---- コールバックに渡すコンテキスト ----------------------------------------

struct TapContext {
    hotkeys: Vec<HotkeyDef>,
    drag_binary: String,
    drop_delay: u64,
    /// タップ自身への参照（再有効化のため）
    tap: CFMachPortRef,
    /// 実行中フラグ（多重起動防止）
    running: Arc<Mutex<bool>>,
}

// CFMachPortRef は *mut c_void なので Send を手動実装
unsafe impl Send for TapContext {}
unsafe impl Sync for TapContext {}

// ---- CGEventTap コールバック ------------------------------------------------

unsafe extern "C" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // タップが無効化された通知 (event_type=31) が来たら再有効化する
    // kCGEventTapDisabledByTimeout = 0xFFFFFFFE, kCGEventTapDisabledByUserInput = 0xFFFFFFFF
    if event_type == 0xFFFFFFFE || event_type == 0xFFFFFFFF {
        let ctx = unsafe { &*(user_info as *const TapContext) };
        eprintln!("[helper] EventTap が無効化されました。再有効化します。");
        unsafe { CGEventTapEnable(ctx.tap, true) };
        return event;
    }

    if event_type != CG_EVENT_KEY_DOWN {
        return event;
    }

    let ctx = unsafe { &*(user_info as *const TapContext) };
    let keycode = unsafe { CGEventGetIntegerValueField(event, CG_KEYBOARD_EVENT_KEYCODE) } as u16;
    let flags = unsafe { CGEventGetFlags(event) };

    // 修飾キーマスク（余分なフラグを除去）
    const MOD_MASK: u64 = 0x0002_0000 | 0x0004_0000 | 0x0008_0000 | 0x0010_0000;
    let flags_masked = flags & MOD_MASK;

    for hk in &ctx.hotkeys {
        if hk.keycode == keycode && hk.modifiers.as_cg_flags() == flags_masked {
            eprintln!(
                "[helper] ホットキー検出: keycode={} → デッキ{}",
                keycode, hk.deck
            );

            // 多重起動チェック
            let running = ctx.running.clone();
            {
                let mut guard = running.lock().unwrap();
                if *guard {
                    eprintln!("[helper] drag-into-djay 実行中のためスキップ");
                    return std::ptr::null_mut();
                }
                *guard = true;
            }

            // 別スレッドで実行してコールバックをすぐに返す
            let binary = ctx.drag_binary.clone();
            let deck = hk.deck;
            let drop_delay = ctx.drop_delay;
            std::thread::spawn(move || {
                invoke_drag(&binary, deck, drop_delay);
                *running.lock().unwrap() = false;
            });

            // イベントを消費
            return std::ptr::null_mut();
        }
    }

    event
}

// ---- drag-into-djay 呼び出し -----------------------------------------------

fn resolve_drag_binary(override_path: &Option<String>) -> String {
    if let Some(p) = override_path {
        return p.clone();
    }
    // 自バイナリと同じディレクトリにある drag-into-djay を使う
    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        exe.push("drag-into-djay");
        if exe.exists() {
            return exe.to_string_lossy().into_owned();
        }
    }
    // フォールバック: PATH から探す
    "drag-into-djay".to_string()
}

fn invoke_drag(binary: &str, deck: u8, drop_delay: u64) {
    let status = std::process::Command::new(binary)
        .args([
            "--deck",
            &deck.to_string(),
            "--drop-delay",
            &drop_delay.to_string(),
        ])
        .status();
    match status {
        Ok(s) if s.success() => eprintln!("[helper] drag-into-djay 完了"),
        Ok(s) => eprintln!("[helper] drag-into-djay 終了コード: {}", s),
        Err(e) => eprintln!("[helper] drag-into-djay 起動失敗: {}", e),
    }
}

// ---- 公開 API --------------------------------------------------------------

pub fn run_event_loop(config: &Config) -> Result<()> {
    let drag_binary = resolve_drag_binary(&config.drag_binary);
    eprintln!("[helper] drag-into-djay パス: {}", drag_binary);

    // tap は後で ctx に入れるため、一旦ダミーで Box を作り raw ポインタを確保してから
    // tap 作成後に書き込む
    let ctx = Box::new(TapContext {
        hotkeys: vec![config.hotkey_deck1.clone(), config.hotkey_deck2.clone()],
        drag_binary,
        drop_delay: config.drop_delay,
        tap: std::ptr::null_mut(), // 後で上書き
        running: Arc::new(Mutex::new(false)),
    });
    let ctx_ptr = Box::into_raw(ctx);

    let tap = unsafe {
        CGEventTapCreate(
            CG_HID_EVENT_TAP,
            CG_HEAD_INSERT_EVENT_TAP,
            CG_EVENT_TAP_OPTION_DEFAULT,
            CG_EVENT_MASK_KEY_DOWN,
            event_tap_callback,
            ctx_ptr as *mut c_void,
        )
    };

    if tap.is_null() {
        unsafe { drop(Box::from_raw(ctx_ptr)) };
        return Err(anyhow!(
            "CGEventTap の作成に失敗しました。\n\
             Accessibility の権限が付与されているか確認してください。\n\
             システム設定 > プライバシーとセキュリティ > アクセシビリティ"
        ));
    }

    // tap ポインタを ctx に書き込む
    unsafe { (*ctx_ptr).tap = tap };

    let source = unsafe { CFMachPortCreateRunLoopSource(std::ptr::null_mut(), tap, 0) };

    unsafe {
        let rl = CFRunLoopGetCurrent();
        CFRunLoopAddSource(rl, source, kCFRunLoopCommonModes);
        eprintln!("[helper] RunLoop 開始");
        CFRunLoopRun();
    }

    unsafe { drop(Box::from_raw(ctx_ptr)) };
    Ok(())
}
