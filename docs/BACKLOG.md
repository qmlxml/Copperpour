# Backlog — deferred work recorded during the fork merge

Each entry names where it was found, what is wrong, and the condition that closes it.

## Live schematic editor (plan step 3, `docs/LIVE_EDITOR.md`)

- **Ten known defects in the Windows-only live editor** (review 2 of step 3; listed in
  `docs/LIVE_EDITOR.md`, "Known defects"). Two can lose design data without any error. The feature is
  off unless `COPPERPOUR_LIVE_EDITOR` is `1`, `on` or `true` (owner ruling 2026-10-31). Closing condition: each defect
  fixed and tested by hand on a Windows build with KiCad 10's Schematic Editor open, on a named
  machine, with the result recorded in `docs/development/`.
- **No Windows CI job.** Copperpour's CI has no job that compiles the Windows-only module, so a
  change that breaks it is not noticed. Closing condition: a CI job that runs
  `cargo check --target` for a Windows target, or a recorded decision not to add one.
- **Windows build not checked.** `crates/copperpour/src/live_editor.rs` was adapted at the merge but
  not compiled for Windows: `cargo check --target x86_64-pc-windows-gnu` needs the MinGW C compiler
  for `nng-sys`. Closing condition: the module compiles for a Windows target.
- **Annotation does not consult the live-editor hook** (`sch_annotate`), so it is refused while the
  editor is open. Closing condition: decided whether it should, and done or documented as intended.

## SES import and DRC (plan steps 2 and 2b)

- **The keep-the-import path of `apply_specctra_ses` is not tested end to end** (needs a kicad-cli
  that fails during a live import with the KiCad editor running). Closing condition: a live test.
- **The live test `specctra_live_undo_test` has not been run since step 2b**; it needs KiCad and
  Freerouting. It now asserts that IPC item IDs appear in the saved candidate, the assumption the
  introduced/preexisting split rests on. Closing condition: run once and passed.
- **Plain `run_drc` reports zero unconnected items without saying that the project set the check to
  "ignore"** (found in step 2b review 3; `apply_specctra_ses` now handles it). Closing condition: the
  same handling in `run_drc`, or a decision that it is intended.
- **`validate_for_manufacturing` cannot show READY for a board designed without a schematic**
  (step 2 review 3). Closing condition: an opt-out argument, or a decision that it is intended.
- **`run_drc` panics on a non-UTF-8 board path** (`pcb.to_str().unwrap()` in cli.rs; predates this
  work). Closing condition: an error instead of a panic.

## kicad-cli processes (plan step 4)

- **A kicad-cli that never exits on a live project runs on with nothing reporting it** after the
  timeout error. It is not killed on purpose, because a killed kicad-cli leaves the project's lock
  file behind. Closing condition: a bounded cleanup that kills it and removes only the lock it
  created, or the hung process reported outward.
- **Manufacturing exports on a staged snapshot are not killed on timeout**
  (`manufacturing.rs`, snapshot mode, through `run_cli`), although the snapshot is a private copy.
  Same as Copperpour before this work. Closing condition: those calls use the private-copy runner.

## SPICE simulation (plan step 4)

- **On Linux and Windows, deck vetting is the only barrier against a schematic's SPICE text running
  commands.** macOS also drops the worker's right to start processes (`RLIMIT_NPROC`); that cannot
  be used on Linux (threads count against it) and nothing replaces it. Measured on Linux: ngspice
  runs a `.control` `shell` command when nothing else stops it. Closing condition: a second barrier
  on Linux (for example a seccomp filter that refuses `execve`/`execveat`, and possibly Landlock for
  file reads) and on Windows (for example a job object limited to one process), each tested.
- **No total limit on kept simulation results.** Twenty runs per project, each up to `max_raw_bytes`
  (256 MiB default, 1 GiB maximum), with no limit across projects; runs for deleted or moved
  projects are never removed. Closing condition: a total limit or a cleanup of runs whose project
  no longer exists.
- **The server reads a whole results file into memory** (up to 1.25 GiB) before parsing it.
  Closing condition: a streaming or bounded read.
- **The worker's own stdout/stderr file (`worker.out`) has no size limit** (the engine log is capped
  at 4 MiB). Closing condition: a cap.
- **A `filesource` reading `/dev/zero` spins until the worker's deadline**, reported only as
  `timed_out`; with the stdio transport this blocks other tool calls for the whole budget. Part of
  the known limit Q17 (data files named in a deck). Closing condition: data-file paths vetted like
  includes, or the cause named in the error.
- **An `.op` analysis cannot be halted early**: it has no pause point, so it runs until the worker's
  own deadline (the budget plus a grace period). Moved here from `docs/SIMULATION_NOTES.md` (known
  open items) by review 3 of step 4. Closing condition: an earlier stop, or the cause named.
- **A code-model path is placed inside an engine command** (`codemodel '<path>'`), against the
  contributor's own recorded rule; the quoting is untested for `$`, `{`, backticks and Windows
  backslashes. Closing condition: tested, or the path passed another way.
- **Between the include size check and the read a file could be swapped** past the 64 MiB inline
  limit. Closing condition: read with a bound instead of checking first.

## Custom-shape pads (plan step 1, not merged)

- **Footprints with custom-shape pads cannot be placed** (about 3.9% of KiCad's library). KiCad's
  typed IPC pad stack has `PssCustom` with `custom_shapes`
  (`crates/copperpour-ipc/proto/board/board_types.proto:294`). Closing condition: the footprint's
  primitives translated into `custom_shapes`, tested against real library footprints.
