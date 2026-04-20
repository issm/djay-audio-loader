#!/usr/bin/env swift

// CGEvent を使ったファイルドラッグ&ドロップの検証
// カーソルを奪わずにイベントを特定プロセスへ送れるか確認する
//
// 使い方:
//   swift .kiro/scripts/verify-cgevent-drag.swift <file_path> <x> <y>
//
// 例:
//   swift .kiro/scripts/verify-cgevent-drag.swift /tmp/test.mp3 700 400

import Cocoa
import CoreGraphics

guard CommandLine.arguments.count == 4 else {
    print("Usage: verify-cgevent-drag.swift <file_path> <x> <y>")
    exit(1)
}

let filePath = CommandLine.arguments[1]
let x = CGFloat(Double(CommandLine.arguments[2])!)
let y = CGFloat(Double(CommandLine.arguments[3])!)

let fileURL = URL(fileURLWithPath: filePath)
guard FileManager.default.fileExists(atPath: filePath) else {
    print("File not found: \(filePath)")
    exit(1)
}

// djay Pro のプロセスを取得
guard let djayApp = NSRunningApplication.runningApplications(
    withBundleIdentifier: "com.algoriddim.djay-iphone-free"
).first else {
    print("djay Pro is not running")
    exit(1)
}

let pid = djayApp.processIdentifier
print("djay Pro PID: \(pid)")

let dropPoint = CGPoint(x: x, y: y)

// NSPasteboard にファイルをセット
let pasteboard = NSPasteboard(name: .drag)
pasteboard.clearContents()
pasteboard.writeObjects([fileURL as NSURL])

// CGEvent でドラッグイベントを送信
// 注: pid 指定でプロセスへ直接送れるか検証
let source = CGEventSource(stateID: .hidSystemState)

func makeMouseEvent(_ type: CGEventType, at point: CGPoint) -> CGEvent? {
    let event = CGEvent(mouseEventSource: source, mouseType: type,
                        mouseCursorPosition: point, mouseButton: .left)
    // 特定プロセスへ送信（カーソルを奪わない方法の検証）
    event?.postToPid(pid)
    return event
}

print("Sending drag events to pid \(pid) at (\(x), \(y))...")

// ドラッグ開始
makeMouseEvent(.leftMouseDown, at: dropPoint)
Thread.sleep(forTimeInterval: 0.05)
makeMouseEvent(.leftMouseDragged, at: dropPoint)
Thread.sleep(forTimeInterval: 0.05)
makeMouseEvent(.leftMouseUp, at: dropPoint)

print("Done. Check if djay Pro loaded the file.")
