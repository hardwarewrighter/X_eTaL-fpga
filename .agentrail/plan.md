# Saga: nibble-machine

Goal: the first demo machine, decided with the user
(docs/calc-approaches.md): a 4-bit machine in the spirit of the
TMS1000, our own design of equivalent size (a 4-bit accumulator, a
small register file, 64 by 4-bit RAM, a ROM of about 1K 8-bit
instructions, keyboard inputs, display outputs), not a replica, with
our own ROM program that makes it a calculator: hexadecimal first,
then octal and decimal. Written in X_eTaL, checked against a pure
X_eTaL reference, then lowered and simulated, sized for the Tang Nano
1K. The slug and the machine's name are working names.

1. isa: docs/machine.md, the registers, memories, instruction set and
   encoding, I/O, timing, sized against the Tang Nano 1K; decided with
   the user.
2. radix-lib: the reference, digit-serial arithmetic in a radix (hex,
   octal, decimal), the specification for the calculator program.
3. machine-lib: the machine in X_eTaL (state vector, ROM, s_tep, run
   harness) and an assembler.
4. hex-calc: the calculator program in ROM, hexadecimal, every test
   key sequence's display equal to the reference's.
5. radixes: octal and decimal; all three against the reference.
6. machine-rtl: hand-lowered to XIR, simulated with the ROM program,
   Verilog pinned, resource estimate against the 1K.
