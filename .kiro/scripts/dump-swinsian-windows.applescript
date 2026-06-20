-- Swinsian の AXWindows をダンプし、ツールチップ表示時の影響を検証する
--
-- 使い方:
--   1. Swinsian の楽曲テーブルで行にマウスオーバーしてツールチップを表示させる
--   2. このスクリプトを実行する:
--      osascript .kiro/scripts/dump-swinsian-windows.applescript
--   3. ツールチップを消した状態でも実行して差分を比較する

tell application "System Events"
	set procs to every process whose bundle identifier is "com.swinsian.Swinsian"
	if procs is {} then
		log "ERROR: Swinsian が起動していません"
		return
	end if

	tell item 1 of procs
		set winCount to count of windows
		log "=== AXWindows 数: " & winCount & " ==="
		log ""

		repeat with i from 1 to winCount
			set win to window i
			try
				set winTitle to title of win
			on error
				set winTitle to "(取得失敗)"
			end try
			try
				set winRole to role of win
			on error
				set winRole to "(取得失敗)"
			end try
			try
				set winSubrole to subrole of win
			on error
				set winSubrole to "(なし)"
			end try
			try
				set winDesc to description of win
			on error
				set winDesc to "(なし)"
			end try
			try
				set winPos to position of win
			on error
				set winPos to "(取得失敗)"
			end try
			try
				set winSize to size of win
			on error
				set winSize to "(取得失敗)"
			end try

			log "--- Window " & i & " ---"
			log "  title:    " & winTitle
			log "  role:     " & winRole
			log "  subrole:  " & winSubrole
			log "  desc:     " & winDesc
			log "  position: " & winPos
			log "  size:     " & winSize

			-- 子要素の role 一覧（最大10個）
			try
				set childElements to UI elements of win
				set childCount to count of childElements
				log "  children: " & childCount & " 個"
				set maxJ to childCount
				if maxJ > 10 then set maxJ to 10
				repeat with j from 1 to maxJ
					set child to item j of childElements
					try
						set childRole to role of child
					on error
						set childRole to "?"
					end try
					try
						set childSubrole to subrole of child
					on error
						set childSubrole to ""
					end try
					try
						set childDesc to description of child
					on error
						set childDesc to ""
					end try
					log "    [" & j & "] role=" & childRole & " subrole=" & childSubrole & " desc=" & childDesc
				end repeat
				if childCount > 10 then
					log "    ... (残り " & (childCount - 10) & " 個)"
				end if
			on error errMsg
				log "  children: 取得失敗 - " & errMsg
			end try
			log ""
		end repeat

		-- AXSelectedRows の確認（メインウィンドウ内テーブル）
		log "=== AXTable / AXSelectedRows 検査 ==="
		set mainWin to window 1
		try
			set tableList to every table of mainWin
			set tableCount to count of tableList
			log "テーブル数: " & tableCount
			repeat with t from 1 to tableCount
				set tbl to item t of tableList
				try
					set selRows to value of attribute "AXSelectedRows" of tbl
					set selCount to count of selRows
					log "  Table " & t & ": 選択行 " & selCount & " 件"
					if selCount > 0 then
						set firstRow to item 1 of selRows
						try
							set rowPos to position of firstRow
							log "    最初の選択行 position: " & rowPos
						on error
							log "    最初の選択行 position: 取得失敗"
						end try
					end if
				on error errMsg
					log "  Table " & t & ": selected rows 取得失敗 - " & errMsg
				end try
			end repeat
		on error errMsg
			log "テーブル取得失敗: " & errMsg
		end try
	end tell
end tell
