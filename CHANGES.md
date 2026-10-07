# Changes

Every commit, newest first, grouped by day. Times are Pacific
(UTC-07:00), as committed.

Categories: `feat` new capability, `fix` a bug or wrong behavior,
`refactor` structure without behavior change, `test` tests only,
`build` build and tooling, `lib` an X_eTaL library or a change to
one, `docs` documentation, `plan` saga planning and reordering,
`release` milestone release, `chore` agentrail bookkeeping (step
complete, saga archive).

## 2026-10-06

- 19:23 `build` Scaffold (saga fpga-foundation, step 1): the process of the sibling repos (agentrail saga, CLAUDE.md with AGENTS.md as a symlink, COPYRIGHT, LICENSE, `.gitignore`, justfile, `scripts/gate.sh`), X_eTaL pinned at 75e6a5c (`XETAL_COMMIT`, `scripts/xetal.sh`, `scripts/check-xetal.sh`, which also checks the Core dump) and X_eTaL-libraries at 8adcd0d for Check, Bits and Strings (`XETAL_LIBRARIES_COMMIT`, `scripts/xetal-libraries.sh`), the library tooling of X_eTaL-ML and X_eTaL-gpu (`scripts/xt`, `libs.py`, `test-libs.sh`, `new-lib.sh`, `run-lib.sh`, `selftest-libs.sh`, `templates/Library`), `docs/research7.txt` (archival), `docs/plan.md` (architecture decisions A1 to A13, the subset in X_eTaL, XIR and hardware, the roadmap: sagas fpga-foundation, fpga-state, fpga-board, fpga-nn, xetal-lowering), `docs/xetal-asks.md` (F1 to F8: the accelerator IR with widths and state, the lowering, sized types and static shapes, and the Core dump are the blockers; this repo carries a provisional IR and the Fixed library until they land), this file, the README.
