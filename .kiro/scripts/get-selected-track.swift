// get-selected-track.swift
// Swinsian / iTunes (Music.app) の選択中トラックのファイルパスを取得する
//
// 優先順位:
//   1. Swinsian または iTunes のうちアクティブ（frontmost）なものを優先
//   2. 両方非アクティブの場合: Swinsian → iTunes の順

import Cocoa
import ApplicationServices

// MARK: - AX ヘルパー

func axValue(_ element: AXUIElement, _ attr: String) -> AnyObject? {
    var value: AnyObject?
    guard AXUIElementCopyAttributeValue(element, attr as CFString, &value) == .success else { return nil }
    return value
}

func axElements(_ element: AXUIElement, _ attr: String) -> [AXUIElement]? {
    return axValue(element, attr) as? [AXUIElement]
}

func axString(_ element: AXUIElement, _ attr: String) -> String? {
    return axValue(element, attr) as? String
}

func axPoint(_ element: AXUIElement, _ attr: String) -> CGPoint? {
    guard let raw = axValue(element, attr) else { return nil }
    var point = CGPoint.zero
    guard AXValueGetValue(raw as! AXValue, .cgPoint, &point) else { return nil }
    return point
}

func axSize(_ element: AXUIElement, _ attr: String) -> CGSize? {
    guard let raw = axValue(element, attr) else { return nil }
    var size = CGSize.zero
    guard AXValueGetValue(raw as! AXValue, .cgSize, &size) else { return nil }
    return size
}

func findAll(_ element: AXUIElement, role: String, depth: Int, result: inout [AXUIElement]) {
    if axString(element, kAXRoleAttribute) == role { result.append(element) }
    guard depth > 0, let children = axElements(element, kAXChildrenAttribute) else { return }
    for child in children { findAll(child, role: role, depth: depth - 1, result: &result) }
}

// MARK: - トラック情報

struct TrackInfo {
    let source: String
    let title: String
    let artist: String
    let album: String
    let duration: String  // 再生時間（例: "3:50"）
    let filePath: String
    let position: CGPoint
    let size: CGSize
}

// セルリストからファイルパスを探す（"/" で始まる値）
func filePathFromCells(_ cells: [AXUIElement]) -> String? {
    for cell in cells {
        let desc = axString(cell, kAXDescriptionAttribute) ?? ""
        if desc.hasPrefix("/") { return desc }
        let val = axString(cell, kAXValueAttribute) ?? ""
        if val.hasPrefix("/") { return val }
        // StaticText の子要素も確認
        if let children = axElements(cell, kAXChildrenAttribute) {
            for child in children {
                let v = axString(child, kAXValueAttribute) ?? ""
                if v.hasPrefix("/") { return v }
            }
        }
    }
    return nil
}

func cellValue(_ cell: AXUIElement) -> String {
    if let children = axElements(cell, kAXChildrenAttribute) {
        for child in children {
            if let v = axString(child, kAXValueAttribute), !v.isEmpty { return v }
        }
    }
    let desc = axString(cell, kAXDescriptionAttribute) ?? ""
    if !desc.isEmpty { return desc }
    return axString(cell, kAXValueAttribute) ?? ""
}

// "03:28" や "1:03:28" 形式を正規化する（分の先頭ゼロを除去）
// 例: "03:28" → "3:28", "01:03:28" → "1:03:28"
func normalizeDuration(_ s: String) -> String {
    let parts = s.components(separatedBy: ":")
    guard parts.count >= 2 else { return s }
    // 先頭パートの先頭ゼロを除去
    let first = String(Int(parts[0]) ?? 0)
    return ([first] + parts.dropFirst()).joined(separator: ":")
}

// MARK: - Swinsian から取得

func getTrackFromSwinsian(_ app: NSRunningApplication) -> TrackInfo? {
    let axApp = AXUIElementCreateApplication(app.processIdentifier)
    guard let windows = axElements(axApp, kAXWindowsAttribute), let window = windows.first else {
        fputs("  [Swinsian] ウィンドウ取得失敗\n", stderr); return nil
    }

    var tables: [AXUIElement] = []
    findAll(window, role: "AXTable", depth: 8, result: &tables)

    for table in tables {
        guard let selectedRows = axElements(table, "AXSelectedRows"), !selectedRows.isEmpty else { continue }
        let row = selectedRows[0]
        guard let pos = axPoint(row, kAXPositionAttribute),
              let sz  = axSize(row, kAXSizeAttribute) else { continue }

        let cells    = axElements(row, kAXChildrenAttribute) ?? []
        let title    = cells.count > 5  ? cellValue(cells[5])  : ""
        let artist   = cells.count > 6  ? cellValue(cells[6])  : ""
        let album    = cells.count > 9  ? cellValue(cells[9])  : ""
        let duration = cells.count > 3  ? normalizeDuration(cellValue(cells[3])) : ""
        let filePath = filePathFromCells(cells) ?? ""  // 取得できない場合は空文字

        return TrackInfo(source: "Swinsian", title: title, artist: artist,
                         album: album, duration: duration, filePath: filePath, position: pos, size: sz)
    }
    fputs("  [Swinsian] 選択行なし or 座標取得失敗\n", stderr)
    return nil
}

// MARK: - iTunes / Music.app から取得

// osascript を subprocess で実行してトラックのメタ情報を取得する
func itunesMetadataViaAppleScript() -> (title: String, artist: String, album: String, duration: String)? {
    let script = """
    tell application "Music"
        set sel to item 1 of (get selection)
        set t  to name of sel
        set ar to artist of sel
        set aa to album artist of sel
        set al to album of sel
        set d  to duration of sel
        return t & "\t" & ar & "\t" & aa & "\t" & al & "\t" & (d as string)
    end tell
    """
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
    proc.arguments = ["-e", script]
    let pipe = Pipe()
    proc.standardOutput = pipe
    proc.standardError = Pipe()
    do {
        try proc.run()
        proc.waitUntilExit()
    } catch {
        return nil
    }
    guard proc.terminationStatus == 0 else { return nil }
    let output = String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8)?
        .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
    let parts = output.components(separatedBy: "\t")
    guard parts.count >= 5 else { return nil }
    let title       = parts[0]
    let artist      = parts[1]
    let albumArtist = parts[2]
    let album       = parts[3]
    // duration は秒数（小数）→ mm:ss に変換
    let totalSec = Int(Double(parts[4]) ?? 0)
    let duration = String(format: "%d:%02d", totalSec / 60, totalSec % 60)
    return (title: title, artist: artist.isEmpty ? albumArtist : artist, album: album, duration: duration)
}

func getTrackFromItunes(_ app: NSRunningApplication) -> TrackInfo? {
    let axApp = AXUIElementCreateApplication(app.processIdentifier)
    guard let windows = axElements(axApp, kAXWindowsAttribute), let window = windows.first else {
        fputs("  [iTunes] ウィンドウ取得失敗\n", stderr); return nil
    }

    // trackTable の選択行から座標を取得
    var tables: [AXUIElement] = []
    findAll(window, role: "AXTable", depth: 10, result: &tables)

    for table in tables {
        let tableId = axString(table, kAXIdentifierAttribute) ?? ""
        guard tableId == "trackTable" else { continue }
        guard let selectedRows = axElements(table, "AXSelectedRows"), !selectedRows.isEmpty else { continue }
        let row = selectedRows[0]
        guard let pos = axPoint(row, kAXPositionAttribute),
              let sz  = axSize(row, kAXSizeAttribute) else { continue }

        let meta     = itunesMetadataViaAppleScript()
        let title    = meta?.title    ?? ""
        let artist   = meta?.artist   ?? ""
        let album    = meta?.album    ?? ""
        let duration = meta?.duration ?? ""

        return TrackInfo(source: "iTunes", title: title, artist: artist,
                         album: album, duration: duration, filePath: "", position: pos, size: sz)
    }
    fputs("  [iTunes] 選択行なし or 座標取得失敗\n", stderr)
    return nil
}

// MARK: - メイン

let swinsian = NSRunningApplication.runningApplications(withBundleIdentifier: "com.swinsian.Swinsian").first
let itunes   = NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.Music").first
    ?? NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.iTunes").first

guard swinsian != nil || itunes != nil else {
    fputs("❌ Swinsian も iTunes も起動していません\n", stderr)
    exit(1)
}

// 優先順位の決定
let candidates: [(NSRunningApplication, (NSRunningApplication) -> TrackInfo?)] = {
    var list: [(NSRunningApplication, (NSRunningApplication) -> TrackInfo?)] = []

    // 1. アクティブなものを先頭に
    if let s = swinsian, s.isActive { list.append((s, getTrackFromSwinsian)) }
    if let i = itunes,   i.isActive { list.append((i, getTrackFromItunes)) }

    // 2. 非アクティブ: Swinsian → iTunes の順
    if let s = swinsian, !s.isActive { list.append((s, getTrackFromSwinsian)) }
    if let i = itunes,   !i.isActive { list.append((i, getTrackFromItunes)) }

    return list
}()

fputs("起動中: \(swinsian != nil ? "Swinsian" : "") \(itunes != nil ? "iTunes" : "")\n", stderr)
fputs("アクティブ: \(swinsian?.isActive == true ? "Swinsian" : "") \(itunes?.isActive == true ? "iTunes" : "")\n", stderr)

for (app, getter) in candidates {
    fputs("試行: \(app.localizedName ?? app.bundleIdentifier ?? "?")\n", stderr)
    if let track = getter(app) {
        print("source:    \(track.source)")
        print("title:     \(track.title)")
        print("artist:    \(track.artist)")
        print("album:     \(track.album)")
        print("duration:  \(track.duration)")
        print("file_path: \(track.filePath)")
        print("position:  \(track.position)")
        print("size:      \(track.size)")
        exit(0)
    }
}

fputs("❌ 選択中トラックが取得できませんでした\n", stderr)
exit(1)
