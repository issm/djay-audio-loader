// drag.rs
// djay Pro の指定デッキの波形エリアへ CGEvent でドラッグ&ドロップを実行する

use anyhow::{Result, anyhow};
use core_foundation::{
    array::CFArray,
    base::{CFRetain, CFType, TCFType},
    string::CFString,
};
use core_graphics::geometry::{CGPoint, CGSize};
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

unsafe fn ax_string(element: AXUIElementRef, attr: &str) -> Option<String> {
    let raw = unsafe { ax_value_raw(element, attr) }?;
    let cf: CFString = unsafe { TCFType::wrap_under_create_rule(raw as _) };
    Some(cf.to_string())
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

unsafe fn ax_size(element: AXUIElementRef, attr: &str) -> Option<CGSize> {
    let raw = unsafe { ax_value_raw(element, attr) }?;
    let mut size = CGSize::new(0.0, 0.0);
    let ok = unsafe {
        AXValueGetValue(
            raw,
            AXValueType::CGSize as u32,
            &mut size as *mut _ as *mut c_void,
        )
    };
    unsafe { CFRelease(raw) };
    if ok { Some(size) } else { None }
}

unsafe fn find_all(
    element: AXUIElementRef,
    role: &str,
    depth: usize,
    result: &mut Vec<AXUIElementRef>,
) {
    if unsafe { ax_string(element, "AXRole") }.as_deref() == Some(role) {
        unsafe { CFRetain(element as _) };
        result.push(element);
    }
    if depth == 0 {
        return;
    }
    if let Some(children) = unsafe { ax_elements(element, "AXChildren") } {
        for child in children {
            unsafe { find_all(child, role, depth - 1, result) };
            unsafe { CFRelease(child as _) };
        }
    }
}

// ---- djay Pro の波形エリア座標取得 -----------------------------------------

pub fn get_waveform_center(deck: u8) -> Result<CGPoint> {
    use objc2_app_kit::NSRunningApplication;
    use objc2_foundation::NSString;

    let pid = {
        let bid = NSString::from_str("com.algoriddim.djay-pro-mac");
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bid);
        match apps.firstObject() {
            Some(app) => app.processIdentifier(),
            None => return Err(anyhow!("djay Pro が起動していません")),
        }
    };

    unsafe {
        let ax_app = AXUIElementCreateApplication(pid);
        let windows = ax_elements(ax_app, "AXWindows")
            .ok_or_else(|| anyhow!("djay Pro のウィンドウ取得失敗"))?;
        let window = *windows
            .first()
            .ok_or_else(|| anyhow!("djay Pro のウィンドウなし"))?;

        let deck_label = format!("デッキ {}", deck);
        let mut buttons: Vec<AXUIElementRef> = Vec::new();
        find_all(window, "AXButton", 6, &mut buttons);

        for &btn in &buttons {
            let desc = ax_string(btn, "AXDescription").unwrap_or_default();
            if desc.contains("波形") && desc.contains(&deck_label) {
                if let (Some(pos), Some(sz)) = (ax_point(btn, "AXPosition"), ax_size(btn, "AXSize"))
                {
                    let cx = pos.x + sz.width / 2.0;
                    let cy = pos.y + sz.height / 2.0;
                    for &b in &buttons {
                        CFRelease(b as _);
                    }
                    for &w in &windows {
                        CFRelease(w as _);
                    }
                    CFRelease(ax_app as _);
                    return Ok(CGPoint::new(cx, cy));
                }
            }
        }

        for &b in &buttons {
            CFRelease(b as _);
        }
        for &w in &windows {
            CFRelease(w as _);
        }
        CFRelease(ax_app as _);
    }

    // フォールバック: 検証ログの固定座標
    let fallback = match deck {
        1 => CGPoint::new(960.0, 253.0),
        2 => CGPoint::new(960.0, 373.0),
        _ => return Err(anyhow!("不正なデッキ番号: {}", deck)),
    };
    eprintln!(
        "警告: 波形エリアが見つからなかったためフォールバック座標を使用: ({}, {})",
        fallback.x, fallback.y
    );
    Ok(fallback)
}

// ---- CGEvent ドラッグ&ドロップ ---------------------------------------------

fn simulate_drag(src: CGPoint, dst: CGPoint) {
    unsafe {
        // MouseDown
        let down = CGEventCreateMouseEvent(
            std::ptr::null_mut(),
            CGEventType::LeftMouseDown as u32,
            src,
            0, // kCGMouseButtonLeft
        );
        CGEventSetIntegerValueField(down, K_CG_MOUSE_EVENT_CLICK_STATE, 1);
        CGEventPost(K_CG_HID_EVENT_TAP, down);
        CFRelease(down as _);
        sleep(Duration::from_millis(50));

        // Drag（中間点を数ステップ挟む）
        let steps = 20usize;
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

pub fn drag_to_djay(track: &TrackInfo, deck: u8) -> Result<()> {
    let src = CGPoint::new(
        track.position.x + track.size.width / 2.0,
        track.position.y + track.size.height / 2.0,
    );
    let dst = get_waveform_center(deck)?;
    eprintln!(
        "ドラッグ: ({:.0}, {:.0}) → ({:.0}, {:.0})",
        src.x, src.y, dst.x, dst.y
    );
    simulate_drag(src, dst);
    Ok(())
}
