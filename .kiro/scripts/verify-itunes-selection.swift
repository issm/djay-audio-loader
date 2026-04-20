// verify-itunes-selection.swift
// Music.app (iTunes) の UI 構造と選択中トラックを調査する

import Cocoa
import ApplicationServices

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

// bundle ID を試す（macOS バージョンによって異なる）
let bundleIds = ["com.apple.Music", "com.apple.iTunes"]
var itunesApp: NSRunningApplication? = nil
var usedBundleId = ""

for bid in bundleIds {
    if let app = NSRunningApplication.runningApplications(withBundleIdentifier: bid).first {
        itunesApp = app
        usedBundleId = bid
        break
    }
}

guard let itunes = itunesApp else {
    print("❌ Music.app / iTunes が起動していません")
    exit(1)
}
print("✅ \(itunes.localizedName ?? "?") (bundle: \(usedBundleId), pid: \(itunes.processIdentifier))")

let axApp = AXUIElementCreateApplication(itunes.processIdentifier)
guard let windows = axElements(axApp, kAXWindowsAttribute), let window = windows.first else {
    print("❌ ウィンドウ取得失敗"); exit(1)
}
print("✅ ウィンドウ: \(axString(window, kAXTitleAttribute) ?? "(no title)")")

// ウィンドウ直下の子要素
print("\n=== ウィンドウ直下の子要素 ===")
if let children = axElements(window, kAXChildrenAttribute) {
    for el in children {
        let role = axString(el, kAXRoleAttribute) ?? "?"
        let id   = axString(el, kAXIdentifierAttribute) ?? ""
        print("  role=\(role) id=\(id)")
    }
}

// AXTable / AXOutline を探す
for role in ["AXTable", "AXOutline"] {
    print("\n=== \(role) を探す ===")
    var elements: [AXUIElement] = []
    findAll(window, role: role, depth: 10, result: &elements)
    print("\(role) 数: \(elements.count)")

    for (i, el) in elements.enumerated() {
        let elId = axString(el, kAXIdentifierAttribute) ?? "(no id)"
        print("\n--- \(role)[\(i)] id=\(elId) ---")

        guard let selectedRows = axElements(el, "AXSelectedRows") else {
            print("  AXSelectedRows: 取得失敗"); continue
        }
        print("  AXSelectedRows: \(selectedRows.count) 行")

        for (rowIdx, row) in selectedRows.enumerated() {
            print("\n  === 選択行[\(rowIdx)] ===")
            if let pos = axPoint(row, kAXPositionAttribute) { print("    position: \(pos)") }
            if let sz  = axSize(row, kAXSizeAttribute)      { print("    size: \(sz)") }
            if let idx = axValue(row, "AXIndex")            { print("    AXIndex: \(idx)") }

            guard let cells = axElements(row, kAXChildrenAttribute) else {
                print("    cells: 取得失敗"); continue
            }
            print("    セル数: \(cells.count)")

            for (cellIdx, cell) in cells.enumerated() {
                let desc = axString(cell, kAXDescriptionAttribute) ?? ""
                let val  = axString(cell, kAXValueAttribute) ?? ""
                var textVal = ""
                if let gc = axElements(cell, kAXChildrenAttribute) {
                    for g in gc {
                        if let v = axString(g, kAXValueAttribute), !v.isEmpty { textVal = v; break }
                    }
                }
                let display = textVal.isEmpty ? (desc.isEmpty ? val : desc) : textVal
                if !display.isEmpty {
                    print("    Cell[\(cellIdx)] desc=\(desc) value=\(display)")
                }
            }
        }
    }
}

// AXFocusedUIElement
print("\n=== AXFocusedUIElement ===")
if let focused = axValue(axApp, kAXFocusedUIElementAttribute) {
    let el = focused as! AXUIElement
    print("  role=\(axString(el, kAXRoleAttribute) ?? "?")")
    print("  id=\(axString(el, kAXIdentifierAttribute) ?? "?")")
}
