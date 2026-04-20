-- djay Pro の UI 要素を列挙し、波形エリア関連の要素を探す
tell application "System Events"
    tell process "djay Pro"
        set win to front window
        set allElements to entire contents of win
        repeat with el in allElements
            try
                set desc to description of el
                set pos to position of el
                set sz to size of el
                set cls to class of el
                if desc contains "波形" or desc contains "waveform" or desc contains "Waveform" then
                    log "=== FOUND ==="
                    log "class: " & cls
                    log "description: " & desc
                    log "position: " & (item 1 of pos) & ", " & (item 2 of pos)
                    log "size: " & (item 1 of sz) & " x " & (item 2 of sz)
                end if
            end try
        end repeat
    end tell
end tell
