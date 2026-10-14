# HEX1000 at the terminal: "just calc" types at the machine, which
# computes every digit in X_eTaL (libs/Hex1000/demos/terminal.xtl).
# Rendered by "just tape" (vhs) into work/tapes/hex1000-calc.gif, which
# the live site shows (scripts/build-pages.sh copies it when present).
Output work/tapes/hex1000-calc.gif

Require just

Set Shell "bash"
Set FontSize 18
Set Width 900
Set Height 420
Set Framerate 12
Set TypingSpeed 180ms
Set Theme "Catppuccin Mocha"
Set Padding 16

Hide
Type "clear"
Enter
Show

Type "just calc"
Enter
Sleep 2s

Type "12+34="
Sleep 1.5s
Type "X"
Sleep 600ms
Type "C0DE+BEEF="
Sleep 1.5s
Type "X"
Sleep 600ms
Type "A*B="
Sleep 1.5s
Type "FFFFFFFF*FFFFFFFF="
Sleep 2.5s
Type "q"
Sleep 1s
