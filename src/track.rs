// track.rs
// Swinsian / Music.app の選択中トラック情報を Accessibility API で取得する
//
// get-selected-track.swift の移植

use anyhow::{Result, anyhow};
use core_foundation::{
    array::CFArray,
    base::{CFRetain, CFType, TCFType},
    string::CFString,
};
use core_graphics::geometry::{CGPoint, CGSize};
use std::ffi::c_void;

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

// ---- プロセス情報 ----------------------------------------------------------

/// バンドル ID からプロセスの PID と isActive を取得する（osascript 経由）
fn find_app(bundle_id: &str) -> Option<(i32, bool)> {
    let script = format!(
        r#"tell application "System Events"
    set procs to every process whose bundle identifier is "{}"
    if procs is {{}} then return ""
    set p to item 1 of procs
    return (unix id of p as string) & "\t" & (frontmost of p as string)
end tell"#,
        bundle_id
    );
    let out = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.split('\t').collect();
    if parts.len() < 2 {
        return None;
    }
    let pid = parts[0].parse::<i32>().ok()?;
    let active = parts[1].trim() == "true";
    Some((pid, active))
}

// ---- AX ヘルパー -----------------------------------------------------------

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

// ---- セルからの値取得 -------------------------------------------------------

unsafe fn cell_value(cell: AXUIElementRef) -> String {
    if let Some(children) = unsafe { ax_elements(cell, "AXChildren") } {
        for &child in &children {
            if let Some(v) = unsafe { ax_string(child, "AXValue") } {
                if !v.is_empty() {
                    for &c in &children {
                        unsafe { CFRelease(c as _) };
                    }
                    return v;
                }
            }
        }
        for &c in &children {
            unsafe { CFRelease(c as _) };
        }
    }
    if let Some(d) = unsafe { ax_string(cell, "AXDescription") } {
        if !d.is_empty() {
            return d;
        }
    }
    unsafe { ax_string(cell, "AXValue") }.unwrap_or_default()
}

unsafe fn file_path_from_cells(cells: &[AXUIElementRef]) -> Option<String> {
    for &cell in cells {
        let desc = unsafe { ax_string(cell, "AXDescription") }.unwrap_or_default();
        if desc.starts_with('/') {
            return Some(desc);
        }
        let val = unsafe { ax_string(cell, "AXValue") }.unwrap_or_default();
        if val.starts_with('/') {
            return Some(val);
        }
        if let Some(children) = unsafe { ax_elements(cell, "AXChildren") } {
            for &child in &children {
                let v = unsafe { ax_string(child, "AXValue") }.unwrap_or_default();
                if v.starts_with('/') {
                    for &c in &children {
                        unsafe { CFRelease(c as _) };
                    }
                    return Some(v);
                }
            }
            for &c in &children {
                unsafe { CFRelease(c as _) };
            }
        }
    }
    None
}

fn normalize_duration(s: &str) -> String {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 {
        return s.to_string();
    }
    let first = parts[0].parse::<u32>().unwrap_or(0).to_string();
    let rest = parts[1..].join(":");
    format!("{}:{}", first, rest)
}

// ---- TrackInfo -------------------------------------------------------------

#[derive(Debug)]
#[allow(dead_code)]
pub struct TrackInfo {
    pub source: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub comment: String,
    pub file_path: String,
    pub position: CGPoint,
    pub size: CGSize,
    /// テーブル（スクロールエリア）の可視領域の位置
    pub table_position: CGPoint,
    /// テーブル（スクロールエリア）の可視領域のサイズ
    pub table_size: CGSize,
}

// ---- Swinsian --------------------------------------------------------------

unsafe fn dump_ax_tree(element: AXUIElementRef, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }
    let indent = "  ".repeat(depth);
    let role = unsafe { ax_string(element, "AXRole") }.unwrap_or_else(|| "?".into());
    let subrole = unsafe { ax_string(element, "AXSubrole") }.unwrap_or_else(|| "".into());
    let title = unsafe { ax_string(element, "AXTitle") }.unwrap_or_else(|| "".into());
    let desc = unsafe { ax_string(element, "AXDescription") }.unwrap_or_else(|| "".into());

    let subrole_str = if subrole.is_empty() {
        String::new()
    } else {
        format!(" subrole={}", subrole)
    };
    let title_str = if title.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", title)
    };
    let desc_str = if desc.is_empty() {
        String::new()
    } else {
        format!(" desc=\"{}\"", desc)
    };

    log::debug!(
        "{}role={}{}{}{}",
        indent,
        role,
        subrole_str,
        title_str,
        desc_str
    );

    if let Some(children) = unsafe { ax_elements(element, "AXChildren") } {
        for child in &children {
            unsafe { dump_ax_tree(*child, depth + 1, max_depth) };
        }
        for &c in &children {
            unsafe { CFRelease(c as _) };
        }
    }
}

unsafe fn get_track_from_swinsian(pid: i32) -> Option<TrackInfo> {
    let ax_app = unsafe { AXUIElementCreateApplication(pid) };
    let windows = unsafe { ax_elements(ax_app, "AXWindows") }?;

    log::debug!("=== Swinsian AXWindows 数: {} ===", windows.len());
    for (i, &win) in windows.iter().enumerate() {
        let role = unsafe { ax_string(win, "AXRole") }.unwrap_or_else(|| "?".into());
        let subrole = unsafe { ax_string(win, "AXSubrole") }.unwrap_or_else(|| "".into());
        let title = unsafe { ax_string(win, "AXTitle") }.unwrap_or_else(|| "".into());
        log::debug!(
            "  Window[{}]: role={} subrole={} title=\"{}\"",
            i,
            role,
            subrole,
            title
        );
    }

    // AXHelpTag（ツールチップ）等を除外し、AXWindow のみを対象とする (#13)
    let window = *windows
        .iter()
        .find(|&&w| unsafe { ax_string(w, "AXRole") }.as_deref() == Some("AXWindow"))?;

    let mut tables: Vec<AXUIElementRef> = Vec::new();
    unsafe { find_all(window, "AXTable", 8, &mut tables) };

    // テーブルを包含する AXScrollArea を探す（可視領域の矩形として使う）
    let mut scroll_areas: Vec<AXUIElementRef> = Vec::new();
    unsafe { find_all(window, "AXScrollArea", 8, &mut scroll_areas) };

    log::debug!("=== AXTable 数: {} ===", tables.len());
    log::debug!("=== AXScrollArea 数: {} ===", scroll_areas.len());

    if tables.is_empty() {
        log::debug!("テーブルが見つからないため、ウィンドウのツリーをダンプします (depth=4):");
        unsafe { dump_ax_tree(window, 0, 4) };
    }

    for (ti, &table) in tables.iter().enumerate() {
        let selected = match unsafe { ax_elements(table, "AXSelectedRows") } {
            Some(r) if !r.is_empty() => {
                log::debug!("  Table[{}]: AXSelectedRows={} 行", ti, r.len());
                r
            }
            Some(r) => {
                log::debug!("  Table[{}]: AXSelectedRows=0 行 (空)", ti);
                drop(r);
                continue;
            }
            _ => {
                log::debug!("  Table[{}]: AXSelectedRows 取得失敗", ti);
                continue;
            }
        };
        let row = selected[0];
        let pos = unsafe { ax_point(row, "AXPosition") }?;
        let sz = unsafe { ax_size(row, "AXSize") }?;

        // テーブルを含む AXScrollArea の可視領域を取得
        let (tbl_pos, tbl_sz) = {
            let mut found = None;
            for &sa in &scroll_areas {
                // AXScrollArea の子に AXTable があるかチェック
                if let Some(children) = unsafe { ax_elements(sa, "AXChildren") } {
                    let has_table = children.iter().any(|&child| {
                        unsafe { ax_string(child, "AXRole") }.as_deref() == Some("AXTable")
                    });
                    for &c in &children {
                        unsafe { CFRelease(c as _) };
                    }
                    if has_table {
                        let p = unsafe { ax_point(sa, "AXPosition") };
                        let s = unsafe { ax_size(sa, "AXSize") };
                        if let (Some(p), Some(s)) = (p, s) {
                            found = Some((p, s));
                            break;
                        }
                    }
                }
            }
            found.unwrap_or((CGPoint::new(pos.x, pos.y), CGSize::new(sz.width, sz.height)))
        };
        log::debug!(
            "  Table[{}]: scroll_area position=({:.0},{:.0}) size=({:.0}x{:.0})",
            ti,
            tbl_pos.x,
            tbl_pos.y,
            tbl_sz.width,
            tbl_sz.height
        );

        let cells = unsafe { ax_elements(row, "AXChildren") }.unwrap_or_default();

        let title = if cells.len() > 5 {
            unsafe { cell_value(cells[5]) }
        } else {
            String::new()
        };
        let artist = if cells.len() > 6 {
            unsafe { cell_value(cells[6]) }
        } else {
            String::new()
        };
        let comment = if cells.len() > 7 {
            unsafe { cell_value(cells[7]) }
        } else {
            String::new()
        };
        let album = if cells.len() > 9 {
            unsafe { cell_value(cells[9]) }
        } else {
            String::new()
        };
        let duration = if cells.len() > 3 {
            normalize_duration(&unsafe { cell_value(cells[3]) })
        } else {
            String::new()
        };
        let file_path = unsafe { file_path_from_cells(&cells) }.unwrap_or_default();

        for &c in &cells {
            unsafe { CFRelease(c as _) };
        }
        for &sa in &scroll_areas {
            unsafe { CFRelease(sa as _) };
        }
        for &t in &tables {
            unsafe { CFRelease(t as _) };
        }
        for &w in &windows {
            unsafe { CFRelease(w as _) };
        }
        unsafe { CFRelease(ax_app as _) };

        return Some(TrackInfo {
            source: "Swinsian".into(),
            title,
            artist,
            album,
            duration,
            comment,
            file_path,
            position: pos,
            size: sz,
            table_position: tbl_pos,
            table_size: tbl_sz,
        });
    }
    None
}

// ---- Music.app -------------------------------------------------------------

fn itunes_metadata_via_applescript() -> Option<(String, String, String, String, String, String)> {
    let script = r#"tell application "Music"
    set sel to item 1 of (get selection)
    set t  to name of sel
    set ar to artist of sel
    set aa to album artist of sel
    set al to album of sel
    set d  to duration of sel
    set c  to comment of sel
    set fp to POSIX path of (location of sel as alias)
    return t & "\t" & ar & "\t" & aa & "\t" & al & "\t" & (d as string) & "\t" & c & "\t" & fp
end tell"#;
    let out = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", script])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let parts: Vec<&str> = s.split('\t').collect();
    if parts.len() < 7 {
        return None;
    }
    let title = parts[0].to_string();
    let artist_raw = parts[1].to_string();
    let album_artist = parts[2].to_string();
    let album = parts[3].to_string();
    let total_sec = parts[4].parse::<f64>().unwrap_or(0.0) as u64;
    let duration = format!("{}:{:02}", total_sec / 60, total_sec % 60);
    let comment = parts[5].to_string();
    let file_path = parts[6].to_string();
    let artist = if artist_raw.is_empty() {
        album_artist
    } else {
        artist_raw
    };
    Some((title, artist, album, duration, comment, file_path))
}

unsafe fn get_track_from_itunes(pid: i32) -> Option<TrackInfo> {
    let ax_app = unsafe { AXUIElementCreateApplication(pid) };
    let windows = unsafe { ax_elements(ax_app, "AXWindows") }?;
    let window = *windows.first()?;

    let mut tables: Vec<AXUIElementRef> = Vec::new();
    unsafe { find_all(window, "AXTable", 10, &mut tables) };

    for &table in &tables {
        let table_id = unsafe { ax_string(table, "AXIdentifier") }.unwrap_or_default();
        if table_id != "trackTable" {
            continue;
        }
        let selected = match unsafe { ax_elements(table, "AXSelectedRows") } {
            Some(r) if !r.is_empty() => r,
            _ => continue,
        };
        let row = selected[0];
        let pos = unsafe { ax_point(row, "AXPosition") }?;
        let sz = unsafe { ax_size(row, "AXSize") }?;

        // テーブルの可視領域の位置・サイズを取得
        let tbl_pos =
            unsafe { ax_point(table, "AXPosition") }.unwrap_or(CGPoint::new(pos.x, pos.y));
        let tbl_sz =
            unsafe { ax_size(table, "AXSize") }.unwrap_or(CGSize::new(sz.width, sz.height));

        let meta = itunes_metadata_via_applescript();
        let title = meta.as_ref().map(|m| m.0.clone()).unwrap_or_default();
        let artist = meta.as_ref().map(|m| m.1.clone()).unwrap_or_default();
        let album = meta.as_ref().map(|m| m.2.clone()).unwrap_or_default();
        let duration = meta.as_ref().map(|m| m.3.clone()).unwrap_or_default();
        let comment = meta.as_ref().map(|m| m.4.clone()).unwrap_or_default();
        let file_path = meta.as_ref().map(|m| m.5.clone()).unwrap_or_default();

        for &t in &tables {
            unsafe { CFRelease(t as _) };
        }
        for &w in &windows {
            unsafe { CFRelease(w as _) };
        }
        unsafe { CFRelease(ax_app as _) };

        return Some(TrackInfo {
            source: "iTunes".into(),
            title,
            artist,
            album,
            duration,
            comment,
            file_path,
            position: pos,
            size: sz,
            table_position: tbl_pos,
            table_size: tbl_sz,
        });
    }
    None
}

// ---- 公開 API --------------------------------------------------------------

pub fn get_selected_track() -> Result<TrackInfo> {
    let swinsian = find_app("com.swinsian.Swinsian");
    let itunes = find_app("com.apple.Music").or_else(|| find_app("com.apple.iTunes"));

    if swinsian.is_none() && itunes.is_none() {
        return Err(anyhow!("Swinsian も Music.app も起動していません"));
    }

    // 優先順位: アクティブなものを先頭に、次に Swinsian → iTunes
    let mut candidates: Vec<(i32, bool)> = Vec::new(); // (pid, is_swinsian)
    if let Some((pid, true)) = swinsian {
        candidates.push((pid, true));
    }
    if let Some((pid, true)) = itunes {
        candidates.push((pid, false));
    }
    if let Some((pid, false)) = swinsian {
        candidates.push((pid, true));
    }
    if let Some((pid, false)) = itunes {
        candidates.push((pid, false));
    }

    unsafe {
        for (pid, is_swinsian) in &candidates {
            let result = if *is_swinsian {
                get_track_from_swinsian(*pid)
            } else {
                get_track_from_itunes(*pid)
            };
            if let Some(track) = result {
                return Ok(track);
            }
        }
    }

    log::debug!(
        "全候補でトラック取得失敗: {:?}",
        candidates
            .iter()
            .map(|(pid, is_sw)| format!(
                "pid={} ({})",
                pid,
                if *is_sw { "Swinsian" } else { "iTunes" }
            ))
            .collect::<Vec<_>>()
    );
    Err(anyhow!("選択中トラックが取得できませんでした"))
}
