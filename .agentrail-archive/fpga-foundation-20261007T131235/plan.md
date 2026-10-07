# Saga: fpga-foundation

Goal: the process, the pinned X_eTaL and X_eTaL-libraries, the
hardware models as tested X_eTaL libraries (fixed-width numbers, the
vector datapath, the calculator), a provisional accelerator IR with
an interpreter, the netlist and its cycle simulator, Verilog for
combinational circuits with a lanes schedule and a reduction tree,
and the first proof: one X_eTaL program's result reproduced by a
simulated circuit, bit for bit. The roadmap and the architecture
decisions are in docs/plan.md; the blockers in docs/xetal-asks.md.

1. scaffold: agentrail saga, CLAUDE.md/AGENTS.md, README, COPYRIGHT,
   LICENSE, .gitignore, justfile, scripts/gate.sh, the pinned X_eTaL
   and X_eTaL-libraries (XETAL_COMMIT, XETAL_LIBRARIES_COMMIT,
   scripts/xetal.sh, xetal-libraries.sh, check-xetal.sh), the library
   tooling (xt, libs.py, test-libs.sh, new-lib.sh, selftest-libs.sh,
   templates/Library), docs/plan.md, docs/xetal-asks.md, CHANGES.md.
2. fixed-lib: libs/Fixed (fx:), fixed-width two's complement numbers
   at a stated width on whole arrays: w_rap, u_wrap, a_dd, s_ub,
   m_ul, n_eg, s_at, o_verflow?, b_its, v_alue, h_ex; tests, pinned
   types, demos, the reference page.
3. datapath-lib: libs/Datapath (dp:), the vector examples at a
   width: v_add, s_axpy, t_hreshold, k_eep, s_um (a reduction tree),
   l_argest, d_ot, p_ipeline; demos vector-add, saxpy, threshold,
   reduce-sum, pipeline with reg-rs baselines (the golden reference
   for the circuits); the reference page.
4. calc-lib: libs/Calc (calc:), a TMS1000-like four-function
   calculator as a state vector and a pure step function: k_eys,
   d_ecode, a_lu, s_tep, r_un, d_isplay; tests 2+3=5, 12+34=46,
   7*8=56, 123-27=96, chains, wrap; demos; the reference page.
5. xir: components/xir, the provisional IR (widths, shapes, ops,
   select, reduce, slice, concat), its text form (parse, print,
   round-trip), checking, the reference interpreter; each Datapath
   demo's .xir twin; scripts/check-equiv.sh; xetal-fpga check and run.
6. verilog: components/rtl, the netlist, the lanes schedule (spatial,
   temporal, tiled) and the reduction tree, the cycle simulator, the
   Verilog-2005 emitter; every twin's Verilog pinned and simulated
   for every schedule; iverilog and Yosys when present; xetal-fpga
   netlist, verilog, sim, explain; check-equiv runs the simulator.
