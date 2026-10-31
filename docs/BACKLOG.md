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

## Custom-shape pads (plan step 1, not merged)

- **Footprints with custom-shape pads cannot be placed** (about 3.9% of KiCad's library). KiCad's
  typed IPC pad stack has `PssCustom` with `custom_shapes`
  (`crates/copperpour-ipc/proto/board/board_types.proto:294`). Closing condition: the footprint's
  primitives translated into `custom_shapes`, tested against real library footprints.
