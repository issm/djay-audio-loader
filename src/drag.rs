// drag.rs
// djay Pro の指定デッキの波形エリアへ CGEvent でドラッグ&ドロップを実行する

use anyhow::{Result, anyhow};
use core_foundation::{
    array::CFArray,
    base::{CFRetain, CFType, TCFType},
    string::CFString,
};
use core_graphics::geometry::CGPoint;
use std::ffi::c_void;
use std::thread::sleep;
use std::time::Duration;

use crate::track::TrackInfo;

// ---- AXUIElement FFI -------------------------------------------------------

#[allow(non_camel_case_types)]
type AXUIElementRef = *mut c_void;

#[repr(u32)]
#[allow(dead_code)]
enum AXValueType {
    CGPoint = 1,
    CGSize = 2,
}

unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: *const c_void,
        value: *mut *mut c_void,
    ) -> i32;
    fn AXValueGetValue(value: *mut c_void, the_type: u32, out: *mut c_void) -> bool;
    fn CFRelease(cf: *const c_void);
}

// ---- CGEvent FFI -----------------------------------------------------------

#[allow(non_camel_case_types)]
type CGEventRef = *mut c_void;
#[allow(non_camel_case_types)]
type CGEventSourceRef = *mut c_void;

#[repr(u32)]
#[allow(dead_code)]
enum CGEventType {
    LeftMouseDown = 1,
    LeftMouseUp = 2,
    LeftMouseDragged = 6,
}

// kCGMouseEventClickState = 100, kCGHIDEventTap = 0
const K_CG_MOUSE_EVENT_CLICK_STATE: i32 = 100;
const K_CG_HID_EVENT_TAP: u32 = 0;

unsafe extern "C" {
    fn CGEventCreateMouseEvent(
        source: CGEventSourceRef,
        mouse_type: u32,
        mouse_cursor_position: CGPoint,
        mouse_button: u64,
    ) -> CGEventRef;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: i32, value: i64);
    fn CGEventPost(tap_location: u32, event: CGEventRef);
}

// ---- AX ヘルパー（drag.rs ローカル）----------------------------------------

unsafe fn ax_value_raw(element: AXUIElementRef, attr: &str) -> Option<*mut c_void> {
    let key = CFString::new(attr);
    let mut value: *mut c_void = std::ptr::null_mut();
    let ret = unsafe {
        AXUIElementCopyAttributeValue(
            element,
            key.as_concrete_TypeRef() as *const c_void,
            &mut value,
        )
    };
    if ret == 0 && !value.is_null() {
        Some(value)
    } else {
        None
    }
}

unsafe fn ax_elements(element: AXUIElementRef, attr: &str) -> Option<Vec<AXUIElementRef>> {
    let raw = unsafe { ax_value_raw(element, attr) }?;
    let arr: CFArray<CFType> = unsafe { TCFType::wrap_under_create_rule(raw as _) };
    let v: Vec<AXUIElementRef> = arr
        .iter()
        .map(|e| e.as_concrete_TypeRef() as *mut c_void)
        .collect();
    for &e in &v {
        unsafe { CFRetain(e as _) };
    }
    Some(v)
}

unsafe fn ax_point(element: AXUIElementRef, attr: &str) -> Option<CGPoint> {
    let raw = unsafe { ax_value_raw(element, attr) }?;
    let mut point = CGPoint::new(0.0, 0.0);
    let ok = unsafe {
        AXValueGetValue(
            raw,
            AXValueType::CGPoint as u32,
            &mut point as *mut _ as *mut c_void,
        )
    };
    unsafe { CFRelease(raw) };
    if ok { Some(point) } else { None }
}

// ---- djay Pro の波形エリア座標取得 -----------------------------------------

// 検証ログ (.kiro/logs/dump-all-elements.log) より確定した
// ウィンドウ左上からの相対オフセット（波形エリア中央）
// ウィンドウ位置 (-18, 43) 時:
//   波形 デッキ1: pos=613,194 size=694x119 → 中央=(960,253)
//   波形 デッキ2: pos=613,314 size=694x119 → 中央=(960,373)
// ウィンドウ相対: x=960-(-18)=978, y1=253-43=210, y2=373-43=330
const WAVEFORM_OFFSET: [(f64, f64); 2] = [
    (978.0, 210.0), // デッキ1
    (978.0, 330.0), // デッキ2
];

pub fn get_waveform_center(deck: u8) -> Result<CGPoint> {
    let deck_idx = match deck {
        1 => 0,
        2 => 1,
        _ => return Err(anyhow!("不正なデッキ番号: {}", deck)),
    };

    let script = r#"tell application "System Events"
    set procs to every process whose bundle identifier is "com.algoriddim.djay-iphone-free"
    if procs is {} then return ""
    return unix id of item 1 of procs as string
end tell"#;
    let out = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| anyhow!("osascript 実行失敗: {}", e))?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        return Err(anyhow!("djay Pro が起動していません"));
    }
    let pid = s
        .parse::<i32>()
        .map_err(|_| anyhow!("PID パース失敗: {}", s))?;

    let win_pos = unsafe {
        let ax_app = AXUIElementCreateApplication(pid);
        let windows = ax_elements(ax_app, "AXWindows")
            .ok_or_else(|| anyhow!("djay Pro のウィンドウ取得失敗"))?;
        let window = *windows
            .first()
            .ok_or_else(|| anyhow!("djay Pro のウィンドウなし"))?;
        let pos = ax_point(window, "AXPosition")
            .ok_or_else(|| anyhow!("djay Pro のウィンドウ座標取得失敗"))?;
        for &w in &windows {
            CFRelease(w as _);
        }
        CFRelease(ax_app as _);
        pos
    };

    let (ox, oy) = WAVEFORM_OFFSET[deck_idx];
    Ok(CGPoint::new(win_pos.x + ox, win_pos.y + oy))
}

// ---- CGEvent ドラッグ&ドロップ ---------------------------------------------

fn activate_djay() {
    std::process::Command::new("/usr/bin/osascript")
        .args(["-e", r#"tell application "System Events" to set frontmost of (first process whose bundle identifier is "com.algoriddim.djay-iphone-free") to true"#])
        .output()
        .ok();
}

fn activate_source(source: &str) {
    let bundle_id = match source {
        "Swinsian" => "com.swinsian.Swinsian",
        "iTunes" => "com.apple.Music",
        _ => return,
    };
    let script = format!(
        r#"tell application "System Events" to set frontmost of (first process whose bundle identifier is "{}") to true"#,
        bundle_id
    );
    std::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .ok();
}

fn simulate_drag(src: CGPoint, dst: CGPoint, source: &str, drop_delay_ms: u64, no_activate: bool) {
    unsafe {
        if !no_activate {
            // ドラッグ元アプリをアクティブにしてから MouseDown
            activate_source(source);
            sleep(Duration::from_millis(200));
        }

        let down = CGEventCreateMouseEvent(
            std::ptr::null_mut(),
            CGEventType::LeftMouseDown as u32,
            src,
            0,
        );
        CGEventSetIntegerValueField(down, K_CG_MOUSE_EVENT_CLICK_STATE, 1);
        CGEventPost(K_CG_HID_EVENT_TAP, down);
        CFRelease(down as _);
        sleep(Duration::from_millis(50));

        // ドラッグ開始後に djay Pro をアクティブにする
        activate_djay();
        sleep(Duration::from_millis(if no_activate { 100 } else { 200 }));

        // Drag（中間点を数ステップ挟む）
        let steps = 10usize;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let mid = CGPoint::new(src.x + (dst.x - src.x) * t, src.y + (dst.y - src.y) * t);
            let drag = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                CGEventType::LeftMouseDragged as u32,
                mid,
                0,
            );
            CGEventPost(K_CG_HID_EVENT_TAP, drag);
            CFRelease(drag as _);
            sleep(Duration::from_millis(10));
        }

        // ドロップ先でホバー待機（djay Pro がドラッグオブジェクトを認識するまで）
        sleep(Duration::from_millis(drop_delay_ms));

        // MouseUp
        let up = CGEventCreateMouseEvent(
            std::ptr::null_mut(),
            CGEventType::LeftMouseUp as u32,
            dst,
            0,
        );
        CGEventPost(K_CG_HID_EVENT_TAP, up);
        CFRelease(up as _);
    }
}

// ---- 公開 API --------------------------------------------------------------

pub fn drag_to_djay(
    track: &TrackInfo,
    deck: u8,
    drop_delay_ms: u64,
    no_activate: bool,
) -> Result<()> {
    let dst = get_waveform_center(deck)?;
    let src = CGPoint::new(
        track.table_position.x + 200.0,
        track.position.y + track.size.height / 2.0,
    );
    log::info!(
        "ドラッグ: ({:.0},{:.0}) → ({:.0},{:.0})  [scroll_area: x={:.0} w={:.0}]",
        src.x,
        src.y,
        dst.x,
        dst.y,
        track.table_position.x,
        track.table_size.width,
    );
    simulate_drag(src, dst, &track.source, drop_delay_ms, no_activate);
    Ok(())
}
