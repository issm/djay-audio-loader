-- verify-swinsian-selection-4.applescript
-- 選択行の詳細構造・ファイルパス取得を検証する

tell application "System Events"
    tell process "Swinsian"
        set w to first window
        set sg to first UI element of w whose role is "AXSplitGroup"
        set sa to first UI element of sg whose role is "AXScrollArea"
        set ol to first UI element of sa whose role is "AXOutline"

        set selRows to value of attribute "AXSelectedRows" of ol
        log "選択行数: " & (count of selRows)

        repeat with rowIdx from 1 to count of selRows
            set r to item rowIdx of selRows
            log ""
            log "=== 選択行 [" & rowIdx & "] ==="

            -- 行の全属性を列挙
            log "--- 行の属性 ---"
            set rowAttrs to {"AXRole", "AXSubrole", "AXValue", "AXTitle", "AXDescription", "AXHelp", "AXIdentifier", "AXPosition", "AXSize", "AXURL", "AXFilename"}
            repeat with attr in rowAttrs
                try
                    set v to value of attribute attr of r
                    log "  " & attr & " = " & (v as string)
                on error
                end try
            end repeat

            -- 行内の全子要素を列挙
            log "--- 行内の子要素 ---"
            set rowChildren to entire contents of r
            log "子要素数: " & (count of rowChildren)
            repeat with el in rowChildren
                try
                    set elRole to value of attribute "AXRole" of el
                    set elAttrs to {"AXValue", "AXTitle", "AXDescription", "AXIdentifier", "AXURL", "AXFilename"}
                    set elInfo to "role=" & elRole
                    repeat with a in elAttrs
                        try
                            set v to value of attribute a of el
                            set elInfo to elInfo & " " & a & "=" & (v as string)
                        on error
                        end try
                    end repeat
                    log "  " & elInfo
                on error
                end try
            end repeat
        end repeat
    end tell
end tell
