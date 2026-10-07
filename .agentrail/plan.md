# Saga: fpga-state

Goal: sequential circuits from X_eTaL step functions: state, input
streams, lookups, ROM and RAM in the provisional IR, the netlist and
the Verilog, and a serial line. What the 4-bit machine of saga 3
(nibble-machine, docs/plan.md) is built from. Nothing here needs a
board; the Verilog goes through iverilog and Yosys when installed.

1. registers: XIR state with its init and next-state binding, an
   input stream (one item per clock), lookup, rom and ram; the
   netlist's registers and memories; the simulator; Verilog; tests and
   a twin of Fixed's counter demo.
2. uart: a UART receiver and transmitter as step functions, clock and
   baud rate as schedule data, simulated end to end with a loopback.
3. testbenches: self-checking testbenches for the sequential examples,
   iverilog and Verilator when present, a VCD per run.
