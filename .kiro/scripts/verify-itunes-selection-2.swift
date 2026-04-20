// verify-itunes-selection-2.swift
// iTunes (Music.app) の選択行の全属性・全セルを詳細調査する

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

func findAll(_ element: AXUIElement, role: String, depth: Int, result: inout [AXUIElement]) {
    if axString(element, kAXRoleAttribute) == role { result.append(element) }
    guard depth > 0, let children = axElements(element, kAXChildrenAttribute) else { return }
    for child in children { findAll(child, role: role, depth: depth - 1, result: &result) }
}

func allAttributeNames(_ element: AXUIElement) -> [String] {
    var names: CFArray?
    guard AXUIElementCopyAttributeNames(element, &names) == .success,
          let arr = names as? [String] else { return [] }
    return arr
}

guard let itunes = NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.Music").first
    ?? NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.iTunes").first else {
    print("❌ iTunes が起動していません"); exit(1)
}

let axApp = AXUIElementCreateApplication(itunes.processIdentifier)
guard let windows = axElements(axApp, kAXWindowsAttribute), let window = windows.first else {
    print("❌ ウィンドウ取得失敗"); exit(1)
}

var tables: [AXUIElement] = []
findAll(window, role: "AXTable", depth: 10, result: &tables)

for table in tables {
    let tableId = axString(table, kAXIdentifierAttribute) ?? "(no id)"
    guard tableId == "trackTable" else { continue }
    print("=== trackTable ===")

    guard let selectedRows = axElements(table, "AXSelectedRows"), !selectedRows.isEmpty else {
        print("選択行なし"); continue
    }
    let row = selectedRows[0]

    // 行の全属性
    print("\n--- 選択行の全属性 ---")
    for attr in allAttributeNames(row) {
        if let v = axValue(row, attr) {
            print("  \(attr) = \(v)")
        }
    }

    // セルの全属性
    print("\n--- セルの全属性 ---")
    guard let cells = axElements(row, kAXChildrenAttribute) else { continue }
    print("セル数: \(cells.count)")
    for (i, cell) in cells.enumerated() {
        print("\n  Cell[\(i)]:")
        for attr in allAttributeNames(cell) {
            if let v = axValue(cell, attr) {
                print("    \(attr) = \(v)")
            }
        }
        if let children = axElements(cell, kAXChildrenAttribute) {
            for (j, child) in children.enumerated() {
                print("    Child[\(j)]:")
                for attr in allAttributeNames(child) {
                    if let v = axValue(child, attr) {
                        print("      \(attr) = \(v)")
                    }
                }
            }
        }
    }

    // AXColumns も確認
    print("\n--- AXColumns ---")
    if let cols = axElements(table, "AXColumns") {
        print("カラム数: \(cols.count)")
        for (i, col) in cols.enumerated() {
            let title = axString(col, kAXTitleAttribute) ?? ""
            let id    = axString(col, kAXIdentifierAttribute) ?? ""
            let desc  = axString(col, kAXDescriptionAttribute) ?? ""
            print("  Col[\(i)] title=\(title) id=\(id) desc=\(desc)")
        }
    }
}
