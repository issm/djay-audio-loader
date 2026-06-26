-- Swinsian の深いネストを掘ってAXTableを見つける
tell application "System Events"
	set procs to every process whose bundle identifier is "com.swinsian.Swinsian"
	set swProc to item 1 of procs
	set wins to every window of swProc
	set win to item 1 of wins

	-- Rust コード上では find_all(window, "AXTable", 8, ...) で見つけているので、
	-- entire contents から table を探す
	log "=== Searching for AXTable via entire contents ==="
	set allTables to every UI element of win whose role is "AXTable"
	log "  direct tables in window: " & (count of allTables)

	-- 代わりにプロセスレベルで全テーブルを探す
	log "=== Searching tables in process ==="
	-- まず再帰的に探索
	set topSplit to first UI element of win whose role is "AXSplitGroup"

	-- topSplit > rightSplit(last splitter group) > innerSplit(first splitter group) > innerSplit2(last splitter group)
	set rightSplit to last splitter group of topSplit
	set innerSplit to first splitter group of rightSplit
	set innerSplit2 to last splitter group of innerSplit

	log "=== innerSplit2 ==="
	set p to position of innerSplit2
	set s to size of innerSplit2
	log "  position=" & (p as string) & " size=" & (s as string)

	set children to every UI element of innerSplit2
	log "  children count: " & (count of children)
	repeat with c in children
		set r to role of c
		set cp to position of c
		set cs to size of c
		log "  " & r & " pos=" & (cp as string) & " size=" & (cs as string)

		try
			set children2 to every UI element of c
			repeat with c2 in children2
				set r2 to role of c2
				set p2 to position of c2
				set s2 to size of c2
				log "    " & r2 & " pos=" & (p2 as string) & " size=" & (s2 as string)
				if r2 is "AXTable" or r2 is "AXScrollArea" then
					log "    *** " & r2 & " ***"
				end if
			end repeat
		end try
	end repeat
end tell
