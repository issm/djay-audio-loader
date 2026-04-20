// verify-itunes-selection-3.swift
// iTunes (Music.app) の選択中トラック情報を AppleScript 経由で取得する

import Cocoa

func runAppleScript(_ source: String) -> String? {
    var error: NSDictionary?
    let script = NSAppleScript(source: source)
    let result = script?.executeAndReturnError(&error)
    if let err = error {
        fputs("AppleScript error: \(err)\n", stderr)
        return nil
    }
    return result?.stringValue
}

// 選択中トラックの各プロパティを取得
let properties = [
    ("name",         "name of sel"),
    ("artist",       "artist of sel"),
    ("albumArtist",  "album artist of sel"),
    ("album",        "album of sel"),
    ("filePath",     "POSIX path of (location of sel)"),
]

print("=== AppleScript で選択中トラック情報を取得 ===")
for (label, expr) in properties {
    let src = """
    tell application "Music"
        set sel to item 1 of (get selection)
        return \(expr)
    end tell
    """
    let val = runAppleScript(src) ?? "(取得失敗)"
    print("  \(label): \(val)")
}
