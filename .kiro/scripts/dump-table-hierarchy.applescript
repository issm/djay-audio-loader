-- Swinsian の AXTable までの階層を辿り、各要素の role, position, size を表示する
-- AXTable の AXParent を再帰的に辿る方式

use framework "Foundation"
use framework "ApplicationServices"
use scripting additions

-- PID 取得
set pid to do shell script "pgrep -x Swinsian | head -1"
set pid to pid as integer

-- AXUIElementCreateApplication
set appRef to current application's NSAccessibilityUnignoredDescendant(missing value)

-- osascript では直接 AXUIElement API を叩けないので、
-- System Events 経由で table の属性を辿る

tell application "System Events"
	set procs to every process whose bundle identifier is "com.swinsian.Swinsian"
	set swProc to item 1 of procs
	set wins to every window of swProc
	set win to item 1 of wins

	-- ウィンドウの全UI要素を再帰的に探す
	log "=== Window children (depth 1) ==="
	set children1 to every UI element of win
	repeat with c in children1
		set r to role of c
		set p to position of c
		set s to size of c
		log "  " & r & " pos=" & (p as string) & " size=" & (s as string)

		-- depth 2
		try
			set children2 to every UI element of c
			repeat with c2 in children2
				set r2 to role of c2
				set p2 to position of c2
				set s2 to size of c2
				log "    " & r2 & " pos=" & (p2 as string) & " size=" & (s2 as string)

				-- depth 3: テーブルを探す
				try
					set children3 to every UI element of c2
					repeat with c3 in children3
						set r3 to role of c3
						if r3 is "AXTable" or r3 is "AXScrollArea" then
							set p3 to position of c3
							set s3 to size of c3
							log "      " & r3 & " pos=" & (p3 as string) & " size=" & (s3 as string)
						end if
					end repeat
				end try
			end repeat
		end try
	end repeat
end tell
