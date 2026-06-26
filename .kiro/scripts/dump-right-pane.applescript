-- Swinsian の右側 SplitGroup 内を掘る
tell application "System Events"
	set procs to every process whose bundle identifier is "com.swinsian.Swinsian"
	set swProc to item 1 of procs
	set wins to every window of swProc
	set win to item 1 of wins

	-- Window > SplitGroup > 右側 SplitGroup (2番目の SplitGroup)
	set topSplit to first UI element of win whose role is "AXSplitGroup"
	set rightSplit to last splitter group of topSplit

	log "=== Right SplitGroup ==="
	set p to position of rightSplit
	set s to size of rightSplit
	log "  position=" & (p as string) & " size=" & (s as string)

	log "=== Right SplitGroup children ==="
	set children to every UI element of rightSplit
	repeat with c in children
		set r to role of c
		set cp to position of c
		set cs to size of c
		log "  " & r & " pos=" & (cp as string) & " size=" & (cs as string)

		-- もう1階層
		try
			set children2 to every UI element of c
			repeat with c2 in children2
				set r2 to role of c2
				set p2 to position of c2
				set s2 to size of c2
				log "    " & r2 & " pos=" & (p2 as string) & " size=" & (s2 as string)

				-- テーブルがあればその親のScrollAreaを特定したい
				if r2 is "AXTable" then
					log "    *** FOUND TABLE ***"
				end if
				if r2 is "AXScrollArea" then
					log "    *** FOUND SCROLL AREA ***"
					try
						set children3 to every UI element of c2
						repeat with c3 in children3
							set r3 to role of c3
							set p3 to position of c3
							set s3 to size of c3
							log "      " & r3 & " pos=" & (p3 as string) & " size=" & (s3 as string)
						end repeat
					end try
				end if
			end repeat
		end try
	end repeat
end tell
