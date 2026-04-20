-- verify-swinsian-selection-3.applescript
-- AXSplitGroup > AXScrollArea > AXOutline の AXSelectedRows を取得する

tell application "System Events"
    tell process "Swinsian"
        set w to first window
        set sg to first UI element of w whose role is "AXSplitGroup"

        log "=== SplitGroup 直下の ScrollArea を列挙 ==="
        set scrollAreas to every UI element of sg whose role is "AXScrollArea"
        log "ScrollArea 数: " & (count of scrollAreas)

        repeat with i from 1 to count of scrollAreas
            set sa to item i of scrollAreas
            try
                set saId to value of attribute "AXIdentifier" of sa
            on error
                set saId to "(no id)"
            end try
            log "  ScrollArea[" & i & "] id=" & saId

            -- Outline を探す
            try
                set ol to first UI element of sa whose role is "AXOutline"
                log "    → AXOutline あり"

                -- AXSelectedRows
                try
                    set selRows to value of attribute "AXSelectedRows" of ol
                    log "    AXSelectedRows: " & (count of selRows) & " 行"
                    repeat with r in selRows
                        log "    --- 選択行 ---"
                        -- 行内のセルを列挙
                        set cells to every UI element of r whose role is "AXCell"
                        repeat with c in cells
                            try
                                set cId to value of attribute "AXIdentifier" of c
                            on error
                                set cId to ""
                            end try
                            -- セル内の StaticText
                            try
                                set txts to every UI element of c whose role is "AXStaticText"
                                repeat with t in txts
                                    try
                                        set tVal to value of attribute "AXValue" of t
                                        log "      cell id=" & cId & " text=" & tVal
                                    on error
                                    end try
                                end repeat
                            on error
                            end try
                        end repeat

                        -- 行の position / size
                        try
                            set rPos to value of attribute "AXPosition" of r
                            set rSz to value of attribute "AXSize" of r
                            log "      position=" & (rPos as string) & " size=" & (rSz as string)
                        on error
                        end try
                    end repeat
                on error e
                    log "    AXSelectedRows エラー: " & e
                end try

            on error
                log "    → AXOutline なし"
            end try
        end repeat

        log ""
        log "=== SplitGroup 内の全 Outline を再帰検索 ==="
        set allOutlines to every outline of sg
        log "Outline 数: " & (count of allOutlines)
    end tell
end tell
