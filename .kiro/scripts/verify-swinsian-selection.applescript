-- verify-swinsian-selection.applescript
-- Swinsian の「現在の選択項目」を Accessibility API で検出する検証スクリプト

tell application "System Events"
    tell process "Swinsian"
        log "=== Swinsian プロセス情報 ==="
        log "frontmost: " & (frontmost as string)

        set wins to windows
        log "ウィンドウ数: " & (count of wins)

        repeat with w in wins
            log "--- ウィンドウ: " & (name of w) & " ---"
            log "  position: " & (position of w as string)
            log "  size: " & (size of w as string)
        end repeat

        log ""
        log "=== 選択項目の検出試行 ==="

        -- 方法1: AXSelectedRows (table/outline ビュー)
        log "--- 方法1: AXSelectedRows ---"
        try
            set tbl to first table of first scroll area of first window
            set selRows to value of attribute "AXSelectedRows" of tbl
            log "AXSelectedRows 取得成功: " & (count of selRows) & " 行"
            repeat with r in selRows
                log "  行: " & (value of attribute "AXValue" of r as string)
            end repeat
        on error e
            log "AXSelectedRows (table): " & e
        end try

        -- 方法2: outline ビュー
        log "--- 方法2: AXSelectedRows (outline) ---"
        try
            set ol to first outline of first scroll area of first window
            set selRows to value of attribute "AXSelectedRows" of ol
            log "AXSelectedRows (outline) 取得成功: " & (count of selRows) & " 行"
            repeat with r in selRows
                log "  行: " & (value of attribute "AXValue" of r as string)
            end repeat
        on error e
            log "AXSelectedRows (outline): " & e
        end try

        -- 方法3: scroll area 内の全要素を列挙
        log "--- 方法3: scroll area 内要素の列挙 ---"
        try
            set sa to first scroll area of first window
            set saChildren to entire contents of sa
            log "scroll area 内要素数: " & (count of saChildren)
            repeat with el in saChildren
                try
                    set elRole to value of attribute "AXRole" of el
                    set elDesc to value of attribute "AXDescription" of el
                    log "  role=" & elRole & " desc=" & elDesc
                on error
                    try
                        set elRole to value of attribute "AXRole" of el
                        log "  role=" & elRole & " (desc取得失敗)"
                    on error
                        log "  (要素情報取得失敗)"
                    end try
                end try
            end repeat
        on error e
            log "scroll area 列挙: " & e
        end try

    end tell
end tell
