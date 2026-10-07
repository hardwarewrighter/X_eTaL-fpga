# Saga: fpga-state

Goal: sequential circuits written in X_eTaL. The calculator of
libs/Calc, a pure step function over a state vector, runs as
registers, a decoder and an ALU in a simulated circuit, with the
evaluator's answers for every test key sequence; then a UART frames
it as "12 + 34 =" in and "46" out. Nothing here needs the board; the
Verilog goes through iverilog and Yosys when they are installed. The
roadmap is in docs/plan.md; the asks in docs/xetal-asks.md (F5: the
left fold a clocked circuit is).

1. registers: XIR state (a `state` value with its init, a `next`
   binding, a `step` over an input stream), `lookup` and `rom`; the
   netlist's registers and memories driven from the program; the
   simulator clocks one input per cycle; Verilog; tests and twins.
2. calc-rtl: the Calc step hand-lowered to XIR; the calculator as RTL
   (state registers, next-state logic, a key input, a display
   output); every key sequence of the Calc tests reproduced by the
   simulated circuit; the Verilog pinned; explain.
3. uart: a UART receiver and transmitter in the netlist (baud rate
   and clock as schedule data); the calculator over UART, simulated
   end to end.
4. testbenches: self-checking testbenches for the sequential
   examples, iverilog and Verilator when present, a VCD per run.
