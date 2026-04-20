-- AXUIElement 経由でデッキ要素に値をセットできるか検証する
-- 事前に find-waveform-elements.applescript で座標を確認しておくこと

tell application "System Events"
    tell process "djay Pro"
        set win to front window
        set allElements to entire contents of win
        repeat with el in allElements
            try
                set desc to description of el
                -- 波形エリアと思われる要素に対して AXValue のセットを試みる
                if desc contains "波形" or desc contains "waveform" or desc contains "Waveform" then
                    log "Trying to set value on: " & desc
                    -- ファイルパスをセット（対応していれば受け付ける）
                    set value of el to "/tmp/test.mp3"
                    log "SUCCESS: value set"
                end if
            on error errMsg
                log "FAILED: " & errMsg
            end try
        end repeat
    end tell
end tell
