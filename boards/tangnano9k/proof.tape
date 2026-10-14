# Live Nano 9K counter/UART proof. This loads SRAM, not flash.
Output work/tapes/nano9k-proof.mp4
Output work/tapes/nano9k-proof.gif
Require just
Set Shell "bash"
Set FontSize 18
Set Width 1200
Set Height 620
Set Framerate 12
Set TypingSpeed 35ms
Set Theme "Catppuccin Mocha"
Set Padding 16
Set WaitTimeout 30s
Hide
Type "clear"
Enter
Show
Type "cat boards/examples/counter.xtl"
Enter
Sleep 3s
Type "just board-probe-check"
Enter
Wait+Screen /PASS: reviewed X_eTaL/
Sleep 3s
Type "just board-probe-test"
Enter
Wait+Screen /PASS: 512 physical counter/
Sleep 6s
