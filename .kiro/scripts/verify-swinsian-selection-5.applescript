-- verify-swinsian-selection-5.applescript
-- 選択行の属性・子要素を UI elements で取得する

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
            set rowAttrs to {"AXRole", "AXSubrole", "AXValue", "AXTitle", "AXDescription", "AXHelp", "AXIdentifier", "AXPosition", "AXSize", "AXURL", "AXFilename", "AXIndex"}
            repeat with attr in rowAttrs
                try
                    set v to value of attribute attr of r
                    log "  " & attr & " = " & (v as string)
                on error e
                    -- log "  " & attr & " → エラー: " & e
                end try
            end repeat

            -- 行内の直接子要素
            log "--- 行内の直接子要素 (UI elements) ---"
            try
                set rowChildren to UI elements of r
                log "直接子要素数: " & (count of rowChildren)
                repeat with el in rowChildren
                    try
                        set elRole to value of attribute "AXRole" of el
                        set elInfo to "role=" & elRole
                        set checkAttrs to {"AXValue", "AXTitle", "AXDescription", "AXIdentifier", "AXURL", "AXFilename", "AXPosition", "AXSize"}
                        repeat with a in checkAttrs
                            try
                                set v to value of attribute a of el
                                set elInfo to elInfo & " | " & a & "=" & (v as string)
                            on error
                            end try
                        end repeat
                        log "  " & elInfo

                        -- さらに子要素を1段掘る
                        try
                            set grandChildren to UI elements of el
                            repeat with gc in grandChildren
                                try
                                    set gcRole to value of attribute "AXRole" of gc
                                    set gcInfo to "    role=" & gcRole
                                    repeat with a in checkAttrs
                                        try
                                            set v to value of attribute a of gc
                                            set gcInfo to gcInfo & " | " & a & "=" & (v as string)
                                        on error
                                        end try
                                    end repeat
                                    log gcInfo
                                on error
                                end try
                            end repeat
                        on error
                        end try
                    on error e
                        log "  (取得失敗: " & e & ")"
                    end try
                end repeat
            on error e
                log "UI elements エラー: " & e
            end try
        end repeat
    end tell
end tell
