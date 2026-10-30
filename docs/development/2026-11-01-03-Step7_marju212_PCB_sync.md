# Step 7: marju212 PCB sync (per behaviour)

Plan: `2026-10-31-00-Fork_Survey.md`, "Step 7 method". Base: `integration` after step 6
(`19fffad`), gate 3,704 passed. The six behaviours judged already present in Copperpour were
checked before work began: `f341fb3`, `10c16e3`, `27b0aec`, `6312362`, `109f890`, `a0583b3` and
`958153e` are ancestors of `integration`, and the tests cited for them exist.

## `a8af6d2`: footprint text tools (cherry-picked)

Conflicts resolved: README and DEV.md to Copperpour's text (counts move at release); the
`pcb_board` registry description, tool registration (beside `footprint_fields` and the zone tools)
and tool-directory rows from both sides; the tool-directory counts left alone. The `pcb_components.rs`
region showed the fork's `a7886a9` code (`LibraryExtras`, `build_footprint_item_with_extras`),
which this step does not take; resolved to Copperpour's `LibraryFootprintData`. `konnect_ipc` paths
renamed. Copperpour's documentation test read the skill's backticked DRC violation types
(`silk_overlap`, `silk_over_copper`) and the `local_x`/`local_y` arguments as tool names; they were
added to its `NOT_TOOLS` list (both arguments are in the tool's schema). Migration note
`docs/api-migrations/footprint-text-tools.md`.

These tools work over KiCad's IPC API, which cannot be run on this machine tonight (no KiCad GUI
session): they are checked by the fork's unit tests only; the fork smoke-tested them against a live
KiCad 10 board.

Gate (`logs/gate-step7-merge.summary`): 3,715 passed (+11, the fork's footprint-text tests), 1
failed (the known baseline failure), 110 ignored.
