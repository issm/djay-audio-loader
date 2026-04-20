-- djay Pro の全 UI 要素を出力する（ドロップゾーン特定用）
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
                -- サイズが 0 の要素は除外
                if (item 1 of sz) > 0 and (item 2 of sz) > 0 then
                    log (cls as string) & " | " & desc & " | pos=" & (item 1 of pos) & "," & (item 2 of pos) & " | size=" & (item 1 of sz) & "x" & (item 2 of sz)
                end if
            end try
        end repeat
    end tell
end tell
