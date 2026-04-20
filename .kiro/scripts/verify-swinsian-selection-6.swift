// verify-swinsian-selection-6.swift
// Swinsian の AXTable（トラックリスト）から選択中トラックを取得する

import Cocoa
import ApplicationServices

func axValue(_ element: AXUIElement, _ attr: String) -> AnyObject? {
    var value: AnyObject?
    let result = AXUIElementCopyAttributeValue(element, attr as CFString, &value)
    guard result == .success else { return nil }
    return value
}

func axValues(_ element: AXUIElement, _ attr: String) -> [AXUIElement]? {
    guard let raw = axValue(element, attr) else { return nil }
    return raw as? [AXUIElement]
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

func findAllElements(_ element: AXUIElement, role: String, depth: Int = 10, result: inout [AXUIElement]) {
    if let r = axString(element, kAXRoleAttribute), r == role { result.append(element) }
    guard depth > 0 else { return }
    guard let children = axValues(element, kAXChildrenAttribute) else { return }
    for child in children {
        findAllElements(child, role: role, depth: depth - 1, result: &result)
    }
}

guard let swinsian = NSRunningApplication.runningApplications(withBundleIdentifier: "com.swinsian.Swinsian").first else {
    print("❌ Swinsian が起動していません"); exit(1)
}
let app = AXUIElementCreateApplication(swinsian.processIdentifier)
guard let windows = axValues(app, kAXWindowsAttribute), let window = windows.first else {
    print("❌ ウィンドウ取得失敗"); exit(1)
}

// AXTable を全て列挙
print("=== AXTable を探す ===")
var tables: [AXUIElement] = []
findAllElements(window, role: "AXTable", depth: 8, result: &tables)
print("Table 数: \(tables.count)")

for (i, table) in tables.enumerated() {
    let tableId = axString(table, kAXIdentifierAttribute) ?? "(no id)"
    print("\n--- Table[\(i)] id=\(tableId) ---")

    guard let selectedRows = axValues(table, "AXSelectedRows") else {
        print("  AXSelectedRows: 取得失敗")
        continue
    }
    print("  AXSelectedRows: \(selectedRows.count) 行")

    for (rowIdx, row) in selectedRows.enumerated() {
        print("\n  === 選択行[\(rowIdx)] ===")

        if let pos = axPoint(row, kAXPositionAttribute) { print("    position: \(pos)") }
        if let sz = axSize(row, kAXSizeAttribute) { print("    size: \(sz)") }
        if let idx = axValue(row, "AXIndex") { print("    AXIndex: \(idx)") }

        // セルを列挙してカラム名と値を取得
        guard let cells = axValues(row, kAXChildrenAttribute) else {
            print("    cells: 取得失敗"); continue
        }
        print("    セル数: \(cells.count)")

        for (cellIdx, cell) in cells.enumerated() {
            let cellId = axString(cell, kAXIdentifierAttribute) ?? ""
            let cellDesc = axString(cell, kAXDescriptionAttribute) ?? ""
            let cellVal = axString(cell, kAXValueAttribute) ?? ""

            // StaticText の値
            var textVal = ""
            if let gcList = axValues(cell, kAXChildrenAttribute) {
                for gc in gcList {
                    if let v = axString(gc, kAXValueAttribute), !v.isEmpty {
                        textVal = v; break
                    }
                }
            }

            let display = textVal.isEmpty ? cellVal : textVal
            if !display.isEmpty || !cellId.isEmpty || !cellDesc.isEmpty {
                print("    Cell[\(cellIdx)] id=\(cellId) desc=\(cellDesc) value=\(display)")
            }
        }
    }
}

// AXFocusedUIElement
print("\n=== AXFocusedUIElement ===")
if let focused = axValue(app, kAXFocusedUIElementAttribute) {
    let el = focused as! AXUIElement
    print("  role=\(axString(el, kAXRoleAttribute) ?? "?")")
    print("  id=\(axString(el, kAXIdentifierAttribute) ?? "?")")
}
