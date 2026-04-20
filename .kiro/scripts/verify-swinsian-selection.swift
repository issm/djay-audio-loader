// verify-swinsian-selection.swift
// Swinsian の選択中アイテムを AXUIElement API で検出する検証スクリプト

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

func findElement(_ element: AXUIElement, role: String, depth: Int = 5) -> AXUIElement? {
    if let r = axString(element, kAXRoleAttribute), r == role { return element }
    guard depth > 0 else { return nil }
    guard let children = axValues(element, kAXChildrenAttribute) else { return nil }
    for child in children {
        if let found = findElement(child, role: role, depth: depth - 1) { return found }
    }
    return nil
}

func findAllElements(_ element: AXUIElement, role: String, depth: Int = 10, result: inout [AXUIElement]) {
    if let r = axString(element, kAXRoleAttribute), r == role { result.append(element) }
    guard depth > 0 else { return }
    guard let children = axValues(element, kAXChildrenAttribute) else { return }
    for child in children {
        findAllElements(child, role: role, depth: depth - 1, result: &result)
    }
}

// Swinsian の PID を取得
guard let swinsian = NSRunningApplication.runningApplications(withBundleIdentifier: "com.swinsian.Swinsian").first else {
    print("❌ Swinsian が起動していません")
    exit(1)
}

let pid = swinsian.processIdentifier
print("✅ Swinsian PID: \(pid)")

let app = AXUIElementCreateApplication(pid)

// ウィンドウ取得
guard let windows = axValues(app, kAXWindowsAttribute), let window = windows.first else {
    print("❌ ウィンドウが取得できません")
    exit(1)
}
print("✅ ウィンドウ取得: \(axString(window, kAXTitleAttribute) ?? "(no title)")")

// AXSplitGroup を探す
print("\n=== AXSplitGroup を探す ===")
var splitGroups: [AXUIElement] = []
findAllElements(window, role: "AXSplitGroup", depth: 3, result: &splitGroups)
print("SplitGroup 数: \(splitGroups.count)")

// AXScrollArea を探す
print("\n=== AXScrollArea を探す ===")
var scrollAreas: [AXUIElement] = []
findAllElements(window, role: "AXScrollArea", depth: 5, result: &scrollAreas)
print("ScrollArea 数: \(scrollAreas.count)")

// AXOutline を探す
print("\n=== AXOutline を探す ===")
var outlines: [AXUIElement] = []
findAllElements(window, role: "AXOutline", depth: 6, result: &outlines)
print("Outline 数: \(outlines.count)")

for (i, outline) in outlines.enumerated() {
    print("\n--- Outline[\(i)] ---")

    // AXSelectedRows
    guard let selectedRows = axValues(outline, "AXSelectedRows") else {
        print("  AXSelectedRows: 取得失敗")
        continue
    }
    print("  AXSelectedRows: \(selectedRows.count) 行")

    for (rowIdx, row) in selectedRows.enumerated() {
        print("\n  === 選択行[\(rowIdx)] ===")

        // 行の各属性
        let rowAttrs = [
            kAXRoleAttribute, kAXSubroleAttribute, kAXValueAttribute,
            kAXTitleAttribute, kAXDescriptionAttribute, kAXHelpAttribute,
            kAXIdentifierAttribute, "AXIndex", "AXURL", "AXFilename"
        ]
        for attr in rowAttrs {
            if let v = axValue(row, attr) {
                print("    \(attr) = \(v)")
            }
        }

        // position / size
        if let pos = axPoint(row, kAXPositionAttribute) {
            print("    AXPosition = \(pos)")
        }
        if let sz = axSize(row, kAXSizeAttribute) {
            print("    AXSize = \(sz)")
        }

        // 子要素 (AXCell) を列挙
        guard let children = axValues(row, kAXChildrenAttribute) else {
            print("    子要素: 取得失敗")
            continue
        }
        print("    子要素数: \(children.count)")

        for (cellIdx, cell) in children.enumerated() {
            let cellRole = axString(cell, kAXRoleAttribute) ?? "?"
            let cellId = axString(cell, kAXIdentifierAttribute) ?? ""
            print("    Cell[\(cellIdx)] role=\(cellRole) id=\(cellId)")

            // セル内の StaticText
            guard let cellChildren = axValues(cell, kAXChildrenAttribute) else { continue }
            for gc in cellChildren {
                let gcRole = axString(gc, kAXRoleAttribute) ?? "?"
                let gcVal = axString(gc, kAXValueAttribute) ?? ""
                let gcId = axString(gc, kAXIdentifierAttribute) ?? ""
                if !gcVal.isEmpty || !gcId.isEmpty {
                    print("      \(gcRole) id=\(gcId) value=\(gcVal)")
                }
            }
        }
    }
}

// AXFocusedUIElement も確認
print("\n=== AXFocusedUIElement ===")
if let focused = axValue(app, kAXFocusedUIElementAttribute) {
    let focusedEl = focused as! AXUIElement
    let role = axString(focusedEl, kAXRoleAttribute) ?? "?"
    print("  role=\(role)")
}
