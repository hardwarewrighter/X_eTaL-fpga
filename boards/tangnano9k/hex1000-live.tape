# Interactive proof against the existing FPGA image: no bitstream load.
Output work/tapes/hex1000-9k-live.mp4
Output work/tapes/hex1000-9k-live.gif
Require just
Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 620
Set Framerate 12
Set TypingSpeed 25ms
Set Theme "Catppuccin Mocha"
Set Padding 20
Set WaitTimeout 15s
Hide
Type "clear"
Enter
Show
Type "just board-calc"
Enter
Wait+Screen /HEX1000>/
Type "X123*4="
Enter
Wait+Screen /48C/
Sleep 500ms
Type "X5-7="
Enter
Wait+Screen /FFFFFFFE/
Sleep 500ms
Type "XFFFFFFFF+1="
Enter
Wait+Screen /FFFFFFFF\+1=.*0/
Sleep 500ms
Type "q"
Enter
Wait+Screen /Serial session closed/
Sleep 2s
