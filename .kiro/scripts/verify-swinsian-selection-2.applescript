-- verify-swinsian-selection-2.applescript
-- Swinsian のウィンドウ直下の UI 構造を列挙する

tell application "System Events"
    tell process "Swinsian"
        set w to first window

        log "=== ウィンドウ直下の子要素 ==="
        set children to UI elements of w
        log "子要素数: " & (count of children)
        repeat with el in children
            try
                set elRole to value of attribute "AXRole" of el
                try
                    set elId to value of attribute "AXIdentifier" of el
                on error
                    set elId to "(no id)"
                end try
                try
                    set elDesc to value of attribute "AXDescription" of el
                on error
                    set elDesc to "(no desc)"
                end try
                log "  role=" & elRole & " id=" & elId & " desc=" & elDesc
            on error e
                log "  (取得失敗: " & e & ")"
            end try
        end repeat

        log ""
        log "=== 全 scroll area を再帰検索 ==="
        set allScrollAreas to every scroll area of w
        log "scroll area 数: " & (count of allScrollAreas)

        log ""
        log "=== entire contents の role 一覧（上位50件）==="
        set allEls to entire contents of w
        log "全要素数: " & (count of allEls)
        set cnt to 0
        repeat with el in allEls
            if cnt > 50 then exit repeat
            try
                set elRole to value of attribute "AXRole" of el
                try
                    set elId to value of attribute "AXIdentifier" of el
                on error
                    set elId to ""
                end try
                if elId is not "" then
                    log "  role=" & elRole & " id=" & elId
                else
                    log "  role=" & elRole
                end if
            on error
            end try
            set cnt to cnt + 1
        end repeat
    end tell
end tell
