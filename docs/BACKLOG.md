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

## Freerouting (plan step 5)

- **No progress over the HTTP transport.** The contribution broadcast progress to every open GET
  event stream (other clients received it); that was removed when merged. Closing condition:
  progress sent on the POST request's own response stream, as the MCP Streamable HTTP
  specification describes.
- **Freerouting's log is looked for only where Freerouting keeps it on Linux** (or at
  `FREEROUTING_LOG`), so on macOS and Windows unrouted connections are never counted and
  `optimize_seconds` never takes effect. Closing condition: the log location on each platform, or a
  count taken from the SES itself.
- **Building the Freerouting command in a unit test creates `~/.local/state/freerouting`** in the
  real home directory. Closing condition: the state directory injected in tests.

- **A slow snapshot download during optimisation ends the whole route** (no missed-poll tolerance on
  `download_running_output`), deleting the temporary SES; and a job that completes between the
  status poll and the download is labelled `OPTIMIZATION_STOPPED` and cancelled. Closing condition:
  the same tolerance as the status polls, and the final state read after the download.
- **The stdio transport does not read stdin while a tool call runs**, so a client that disconnects
  or sends `notifications/cancelled` is not noticed; a routing call then runs to its overall
  timeout (up to 86,400 s) with nothing reported. Predates this work. Closing condition: stdin read
  during calls, with cancellation honoured.
- **The temporary `.konnect-*.tmp.ses` is left behind** when a routing call is dropped, or on a read
  or parse failure in `route_local`. Closing condition: removed on every exit.
- **Two concurrent routing calls share Freerouting's fixed ports and log file**; a log rollover (at
  50 MB) by one loses the other's lines. Closing condition: per-call ports and log.
- **The bottom-side CPL formula has no bottom-side observation behind it.** Bottom-side parts with a
  non-zero correction are now listed for review (plan step 5, review 2). Closing condition: a
  bottom-side observation per rule, recorded in the policy.

## Schematic sheets and templates (plan step 6)

- **No response field says a sheet's position or size was snapped to the grid**, and snapping only
  the size of an off-grid sheet leaves its right and bottom edges off the grid. Closing condition: a
  `snapped` field, and all four edges on the grid.
- **`apply_template` wires pins by reference name**, so a `ref_start` that collides with an existing
  part also wires that part's pins. Closing condition: the refusal of a colliding `ref_start`, or
  wiring by the placed instances' UUIDs.
- **`export_netlist_summary` with `schematic_scope: "hierarchy"` runs kicad-cli on the live project**,
  which creates a `.kicad_prl` file there (measured; no lock left); the file scope is a pure read.
  Closing condition: run on a private copy, as the simulation's export does.
- **The hierarchy summary matches pins to parts by reference alone**, so two parts with the same
  reference (unannotated `R?`) give the second `pin_count: 0`. Closing condition: matched by the
  netlist's sheet path as well.
- **Moving an off-grid sheet keeps its pins' offsets along the edge**, so those pins stay off the
  grid. Closing condition: pins snapped with the sheet.
- **Sheet pins are not put on the grid**: `add_sheet_pin` and `edit_sheet_pin` with `side` alone do not
  snap a pin's place along the edge, while `add_wire` snaps to 1.27 mm and cannot reach it. Closing
  condition: pin positions snapped or refused off the grid.
- **A pooled global-rail verdict does not list what it covered**: per-sheet results still show the
  findings the pooled top level removed, and a rail judged fine has no record of which sheets
  supplied it; rails are pooled by name, so a rail named differently on two sheets is not pooled.
  Closing condition: a pooled `rail_coverage` with its sources.
- **`f517fd8` and `d2961f4` from wilfredo-a14/agent-hub are not merged** (activity reporting to,
  and editor control of, a modified KiCad build). Owner ruling 2026-11-01, with a plan agreed by two
  models: measure first, with stock KiCad 10.0.7 running and a board open, whether KiCad answers
  `GetActiveLayer`, `SetActiveLayer`, `GetVisibleLayers`, `SetVisibleLayers` and `RunAction`
  (`common.Control.zoomFitScreen`); correct `tool-directory.md`'s `set_active_layer` row or the tool
  by the result. Only if they are handled: a new narrow implementation (not a cherry-pick) of
  `set_active_layer` (same name; refusal kept when KiCad answers unsupported; migration note),
  `set_layer_visibility` (the fork's arguments) and `run_editor_action` (PCB editor, fixed list of
  actions that open no dialog), with the full review cycle. Never taken: the activity reporting,
  `activity-hook`, the custom IPC messages, the dispatch-path and live-editor changes, `d2961f4`.
  Revisit the rest if the owner adopts the modified KiCad build. Waiting for: a live KiCad session.

## PCB sync and library data (plan step 7)

- **A pad's reach ignores a pad-stack offset**: KiCad's IPC `PadStack` and `PadStackLayer` messages
  carry no offset or trapezoid delta, so a pad whose copper is offset from its position is judged by
  a circle around its position. Closing condition: measured on a live board with an offset pad.
- **A library that drops its `zone_connect` clause leaves the board footprint's earlier override in
  place** (KiCad's refresh would reset it to inherited). Closing condition: decided against KiCad's
  behaviour on a live board.
- **Footprint-local coordinates on a back-side footprint are not supported** by the footprint text
  tools (`local` is null, local moves refused) and are converted by rotation alone, without the
  mirror, by Copperpour's footprint graphics tools (`list_footprint_graphics`,
  `set_footprint_graphic_points`; this predates step 7). Closing condition: KiCad's back-side
  transform measured on a live board and applied in both.
- **A net-tie pad name containing white space** is one name to KiCad's DRC but two to KiCad's IPC
  writer (`footprint.cpp` 10.0.7, lines 369 and 3862), so a library refresh of such a footprint would
  never be a no-op. Copperpour follows the DRC reading. Closing condition: measured on a live board, or
  such names refused.
- **`set_pad_net` changes a pad's net without the copper check** the sync applies, so it can make the
  short the sync refuses (predates step 7). Closing condition: the same check, with an explicit
  override argument.
- **No tool lists or deletes an arc track on a live board**, so a sync refused by an arc, by an
  unreadable zone, or by copper whose geometry could not be read can only be cleared in KiCad (the
  refusal says so). Closing condition: `query_traces` and `delete_trace` cover arcs.
- **`kept_zone_connection` is reported only on a planned update**: a footprint whose only difference
  from its library is the zone connection is counted unchanged and not reported. Closing condition:
  reported for unchanged footprints too, or an option to take the library's design-rule overrides as
  KiCad's "Update clearance overrides" does.
- **The sync's readback does not check that net ties were kept**, as it does for models and
  attributes. Closing condition: `net_ties` compared in the readback.
- **The IPC paths of this step were not run against KiCad here** (no KiCad GUI session): the footprint
  text tools, net ties on placement, and the refresh's no-op with net ties rely on unit tests and on
  the fork's live checks. Closing condition: one live run on a board with a solder jumper.

## Custom-shape pads (plan step 1, not merged)

- **Footprints with custom-shape pads cannot be placed** (about 3.9% of KiCad's library). KiCad's
  typed IPC pad stack has `PssCustom` with `custom_shapes`
  (`crates/copperpour-ipc/proto/board/board_types.proto:294`). Closing condition: the footprint's
  primitives translated into `custom_shapes`, tested against real library footprints.
