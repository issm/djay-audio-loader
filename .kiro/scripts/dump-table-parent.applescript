-- Swinsian の AXTable の親要素（AXScrollArea）の位置・サイズを調べる
tell application "System Events"
	set procs to every process whose bundle identifier is "com.swinsian.Swinsian"
	if procs is {} then
		error "Swinsian が起動していません"
	end if
	set swProc to item 1 of procs
	set wins to every window of swProc
	if wins is {} then
		error "ウィンドウなし"
	end if
	set win to item 1 of wins

	-- ウィンドウ情報
	log "=== Window ==="
	log "  position: " & (position of win as string)
	log "  size: " & (size of win as string)

	-- AXScrollArea を探す
	set scrollAreas to every scroll area of win
	log "=== ScrollAreas: " & (count of scrollAreas) & " ==="
	repeat with i from 1 to count of scrollAreas
		set sa to item i of scrollAreas
		set saPos to position of sa
		set saSize to size of sa
		set saRole to role of sa
		log "  ScrollArea[" & i & "]: role=" & saRole & " position=" & (saPos as string) & " size=" & (saSize as string)

		-- その中のテーブルを探す
		set tbls to every table of sa
		if (count of tbls) > 0 then
			repeat with j from 1 to count of tbls
				set tbl to item j of tbls
				set tblPos to position of tbl
				set tblSize to size of tbl
				log "    Table[" & j & "]: position=" & (tblPos as string) & " size=" & (tblSize as string)
			end repeat
		end if
	end repeat
end tell
