; HEX1000's calculator program: an eight-digit hexadecimal calculator
; with + - * = and X (clear), the behavior of the Radix library
; (libs/Radix) key for key. Assembled by hx:a_ssemble; the library
; includes this file as hx:Calc.
;
; RAM: file 0 the accumulator, file 1 the entry, file 2 the
; multiplier's digits while multiplying, each eight words, word 0 the
; least significant digit. File 3 holds the control words:
;   word 0  the column of the key just read (8 to 13)
;   word 1  the pending operation: 0 none, 1 +, 2 -, 3 *
;   word 2  the phase: 0 after an operator, 1 entering digits, 2 after =
;   word 3  the digit just keyed
;   word 4  the operator key's row: 0 +, 1 -, 2 *, 3 =
;   word 5  the multiplier digit being worked on
;   words 8 to 13: the K bits read from that column
;
; The R latches: R0 to R7 strobe the display's digits (digit 0 on the
; right): a digit takes O when its latch rises; O's bit 4 (S at TDO)
; set shows the digit A, clear blanks it. R8 to R13 select the key
; columns: the key with code c is in column c / 4 and drives K's bit
; c mod 4 while its column is selected (codes as Radix's: 0 to 15 the
; digits, 16 + 17 - 18 * 19 =, 20 X). R15 is 1 while the program waits
; for a key, so a terminal (or the test harness) types the next key
; only then.
;
; One subroutine level: the subroutines (pages 4 to 6) call nothing;
; the main code (pages 0 to 3) calls them, after LDP of their page.

PAGE 0          ; reset, the key scan, what kind of key

init:   LDX 1
        LDP show
        CALL show       ; show the entry: 0
ready:  TCY 8           ; wait until every key is up
rl:     SETR
        KNEZ
        BR held
        RSTR
        IYC
        YNEC 14
        BR rl
        TCY 15
        SETR            ; R15: ready for a key
scan:   TCY 8
col:    SETR            ; select a column
        KNEZ
        BR hit          ; a key in it
        RSTR
        IYC
        YNEC 14
        BR col
        BR scan
held:   RSTR
        BR ready
hit:    TKA             ; the key's row bit
        RSTR
        LDX 3
        TAM             ; kept in file 3 at the column's word
        TYA
        TCY 0
        TAM             ; the column
        TCY 15
        RSTR            ; R15 0: busy
        TCY 0
        TMY             ; Y: the column again
        CLA             ; A: the row, from the bit
        TBIT 0
        BR rowok
        IA
        TBIT 1
        BR rowok
        IA
        TBIT 2
        BR rowok
        IA
rowok:  YNEC 12
        BR notop
        LDP opkey       ; column 12: + - * =
        BR opkey
notop:  YNEC 13
        BR digit
        ACC 15          ; column 13: only row 0 is a key, X
        BR ready
        LDP clear
        BR clear
digit:  YNEC 8          ; the digit: the row plus 4 per column
        BR dm
        LDP dgo
        BR dgo
dm:     ACC 4
        DYN
        BR digit

PAGE 1          ; a digit key

dgo:    LDX 3
        TCY 3
        TAM             ; keep the digit
        TCY 2
        TMA
        ACC 14
        BR fresh        ; phase 2: a new calculation
        TMA
        ACC 15
        BR app          ; phase 1: append
        LDX 1           ; phase 0: a new entry
        LDP zero
        CALL zero
        BR app2
fresh:  LDX 0
        LDP zero
        CALL zero
        LDX 1
        LDP zero
        CALL zero
        LDX 3
        TCY 1
        TCMIY 0         ; nothing pending
        BR app2
app:    LDX 1
        TCY 7
        MNEZ
        BR full         ; eight digits already: ignored
app2:   LDX 1
        LDP shl
        CALL shl        ; the entry times 16
        LDX 3
        TCY 3
        TMA
        LDX 1
        TCY 0
        TAM             ; plus the digit
        LDX 3
        TCY 2
        TCMIY 1         ; phase 1
        LDX 1
        LDP show
        CALL show       ; show the entry
        LDP ready
        BR ready
full:   LDP ready
        BR ready

PAGE 2          ; an operator key, or =: settle the pending operation

opkey:  LDX 3
        TCY 4
        TAM             ; keep the row
        TCY 2
        TMA
        ACC 14
        BR settled      ; phase 2: nothing keyed since =
        TMA
        ACC 15
        BR p1           ; phase 1: settle
        BR settled      ; phase 0: nothing keyed since the operator
p1:     TCY 1
        TMA             ; the pending operation
        ACC 15
        BR doop
        LDP copy10
        CALL copy10     ; none: the entry is the result
        BR settled
doop:   ACC 15
        BR nadd
        LDP add
        CALL add
        BR settled
nadd:   ACC 15
        BR domul
        LDP sub
        CALL sub
        BR settled
domul:  LDP mul
        BR mul
settled: LDX 3
        TCY 4
        TMA
        ACC 13
        BR equals       ; row 3: =
        TMA
        IA
        TCY 1
        TAM             ; pending: the row plus 1
        TCY 2
        TCMIY 0         ; phase 0
        BR cle
equals: TCY 1
        TCMIY 0         ; nothing pending
        TCMIY 2         ; phase 2
cle:    LDX 1
        LDP zero
        CALL zero       ; the entry starts at 0
        LDX 0
        LDP show
        CALL show       ; show the result
        LDP ready
        BR ready

PAGE 3          ; multiply, and clear

mul:    LDP copy12
        CALL copy12     ; the multiplier (the entry) to file 2
        LDP copy01
        CALL copy01     ; the multiplicand (the accumulator) to file 1
        LDX 0
        LDP zero
        CALL zero       ; the product
        LDX 3
        TCY 5
        TCMIY 7         ; from the multiplier's top digit
mloop:  LDX 0
        LDP shl
        CALL shl        ; the product times 16
inner:  LDX 3
        TCY 5
        TMY
        LDX 2
        MNEZ
        BR mdo          ; this digit not yet used up
        LDX 3
        TCY 5
        DMAN
        BR mnext        ; the next digit down
        LDP settled
        BR settled      ; done: the product is in file 0
mnext:  TAM
        BR mloop
mdo:    DMAN
        TAM             ; one fewer
        LDP add
        CALL add        ; the product plus the multiplicand
        BR inner
clear:  LDX 0
        LDP zero
        CALL zero
        LDX 1
        LDP zero
        CALL zero
        LDX 3
        TCY 1
        TCMIY 0         ; nothing pending
        TCMIY 0         ; phase 0
        LDX 1
        LDP show
        CALL show
        LDP ready
        BR ready

PAGE 4          ; subroutines: show, shl, zero (on file X)

show:   TCY 7           ; from the top digit: leading zeros blank
lead:   TMA
        MNEZ
        BR digs         ; the first digit that is not 0
        YNEC 0
        BR blank
        TDO             ; the number is 0: its last digit shows 0
        SETR
        RSTR
        RETN
blank:  MNEZ            ; S 0: blank
        TDO
        SETR
        RSTR
        DYN
        BR lead
digs:   TMA
        TDO             ; S 1: shown
        SETR
        RSTR
        DYN
        BR digs
        RETN
shl:    TCY 6           ; each word up one, 0 into word 0
sl:     TMA
        IYC
        TAM
        DYN
        DYN
        BR sl
        TCY 0
        TCMIY 0
        RETN
zero:   TCY 0
zl:     TCMIY 0
        YNEC 8
        BR zl
        RETN

PAGE 5          ; subroutines: file 0 plus file 1, file 0 minus file 1

add:    TCY 0
an:     LDX 1           ; no carry in
        TMA
        LDX 0
        AMAAC
        BR ac
        TAMIY
        YNEC 8
        BR an
        RETN
ac:     TAMIY           ; a carry out
        YNEC 8
        BR ci
        RETN
ci:     LDX 1           ; a carry in
        TMA
        LDX 0
        AMAAC
        BR cc
        ACC 1
        BR ac
        TAMIY
        YNEC 8
        BR an
        RETN
cc:     IA
        BR ac
sub:    TCY 0
sn:     LDX 1           ; no borrow in
        TMA
        LDX 0
        SAMAN
        BR snb
        TAMIY           ; a borrow out
        YNEC 8
        BR sb
        RETN
snb:    TAMIY
        YNEC 8
        BR sn
        RETN
sb:     LDX 1           ; a borrow in
        TMA
        LDX 0
        SAMAN
        BR sb1
        ACC 15
        TAMIY
        YNEC 8
        BR sb
        RETN
sb1:    ACC 15
        BR snb
        TAMIY
        YNEC 8
        BR sb
        RETN

PAGE 6          ; subroutines: copies between files

copy10: TCY 0           ; file 1 to file 0
c10:    LDX 1
        TMA
        LDX 0
        TAMIY
        YNEC 8
        BR c10
        RETN
copy12: TCY 0           ; file 1 to file 2
c12:    LDX 1
        TMA
        LDX 2
        TAMIY
        YNEC 8
        BR c12
        RETN
copy01: TCY 0           ; file 0 to file 1
c01:    LDX 0
        TMA
        LDX 1
        TAMIY
        YNEC 8
        BR c01
        RETN
