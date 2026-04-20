// drag-file-to-point.swift
// NSPasteboard にファイル URL を書き込み、CGEvent でドラッグ&ドロップする
//
// Usage: swift drag-file-to-point.swift <file_path> <src_x> <src_y> <dst_x> <dst_y> [drop_delay_ms]

import Cocoa

guard CommandLine.arguments.count >= 6 else {
    fputs("Usage: drag-file-to-point <file_path> <src_x> <src_y> <dst_x> <dst_y> [drop_delay_ms]\n", stderr)
    exit(1)
}

let filePath    = CommandLine.arguments[1]
let srcX        = CGFloat(Double(CommandLine.arguments[2])!)
let srcY        = CGFloat(Double(CommandLine.arguments[3])!)
let dstX        = CGFloat(Double(CommandLine.arguments[4])!)
let dstY        = CGFloat(Double(CommandLine.arguments[5])!)
let dropDelayMs = CommandLine.arguments.count >= 7 ? Int(CommandLine.arguments[6])! : 250

let fileURL = URL(fileURLWithPath: filePath)
guard FileManager.default.fileExists(atPath: filePath) else {
    fputs("❌ ファイルが存在しません: \(filePath)\n", stderr)
    exit(1)
}

// NSPasteboard にファイル URL を書き込む
let pb = NSPasteboard.general
pb.clearContents()
pb.writeObjects([fileURL as NSURL])
fputs("[drag-file] pasteboard set: \(filePath)\n", stderr)

let src = CGPoint(x: srcX, y: srcY)
let dst = CGPoint(x: dstX, y: dstY)

func postMouseEvent(type: CGEventType, pos: CGPoint) {
    let e = CGEvent(mouseEventSource: nil, mouseType: type,
                    mouseCursorPosition: pos, mouseButton: .left)!
    if type == .leftMouseDown {
        e.setIntegerValueField(.mouseEventClickState, value: 1)
    }
    e.post(tap: .cghidEventTap)
}

// MouseDown
postMouseEvent(type: .leftMouseDown, pos: src)
fputs("[drag-file] MouseDown at (\(src.x), \(src.y))\n", stderr)
Thread.sleep(forTimeInterval: 0.05)

// djay Pro をアクティブに
let activateScript = """
tell application "System Events" to set frontmost of (first process whose bundle identifier is "com.algoriddim.djay-iphone-free") to true
"""
if let script = NSAppleScript(source: activateScript) {
    var err: NSDictionary?
    script.executeAndReturnError(&err)
}
Thread.sleep(forTimeInterval: 0.2)
fputs("[drag-file] djay Pro activated\n", stderr)

// Drag（20ステップ）
let steps = 20
for i in 1...steps {
    let t = CGFloat(i) / CGFloat(steps)
    let mid = CGPoint(x: srcX + (dstX - srcX) * t, y: srcY + (dstY - srcY) * t)
    postMouseEvent(type: .leftMouseDragged, pos: mid)
    Thread.sleep(forTimeInterval: 0.01)
}
fputs("[drag-file] dragged to (\(dst.x), \(dst.y))\n", stderr)

// ホバー待機
Thread.sleep(forTimeInterval: Double(dropDelayMs) / 1000.0)

// MouseUp
postMouseEvent(type: .leftMouseUp, pos: dst)
fputs("[drag-file] MouseUp at (\(dst.x), \(dst.y))\n", stderr)
