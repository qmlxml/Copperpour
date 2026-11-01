# Backlog — deferred work recorded during the fork merge

Each entry names where it was found, what is wrong, and the condition that closes it.

## Live schematic editor (plan step 3, `docs/LIVE_EDITOR.md`)

- **Ten known defects in the Windows-only live editor** (review 2 of step 3; listed in
  `docs/LIVE_EDITOR.md`, "Known defects"). Two can lose design data without any error. The feature is
  off unless `COPPERPOUR_LIVE_EDITOR` is `1`, `on` or `true` (owner ruling 2026-10-31). Closing condition: each defect
  fixed and tested by hand on a Windows build with KiCad 10's Schematic Editor open, on a named
  machine, with the result recorded in `docs/development/`.
- **~~No Windows CI job.~~ Corrected 2026-11-01:** Copperpour's CI `check` job runs
  `cargo check --workspace` and the unit and integration tests on `windows-latest` as well as Linux
  and macOS (`.github/workflows/ci.yml`, job `check`, matrix `os`), which compiles the Windows-only
  module. It runs only on a push to `main` or a pull request into `main`
  (`ci.yml` lines 3-7), and only once Actions is enabled on the fork; pushing `integration` alone runs
  nothing.
- **Windows build not checked.** `crates/copperpour/src/live_editor.rs` was adapted at the merge but
  not compiled for Windows: `cargo check --target x86_64-pc-windows-gnu` needs the MinGW C compiler
  for `nng-sys`. Owner ruling 2026-11-01: check it through GitHub CI (push `integration`, enable
  Actions on `qmlxml/Copperpour`, open a pull request from `integration` into the fork's `main`), not
  a local MinGW install. Closing condition: the Windows job of that run passes.
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
- **`apply_template` has no dry run**, so `joined_existing_nets` (template nets that joined existing
  nets of the same name, e.g. a 6 V `VIN` joining a 24 V `VIN`) is first seen after the labels are
  written, and undoing the join is by hand. Closing condition: a preview call that reports the
  joins before anything is written (owner review of overnight decision 13, 2026-11-01).
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
  Revisit the rest if the owner adopts the modified KiCad build.
  **Measured 2026-11-01** on stock KiCad 10.0.7 (Arch package) with the pic_programmer demo, by the
  ignored test `stock_editor_command_probe` in `crates/copperpour-ipc/src/client.rs`:
  `GetActiveLayer` and `SetActiveLayer` are handled, and a change reads back (3 → 34 → restored), so
  `tool-directory.md`'s claim that the stable protocol has no supported active-layer mutation is
  wrong; `RunAction` `common.Control.zoomFitScreen` is handled (`RAS_OK`). **`GetVisibleLayers`
  fails an internal KiCad assertion** (`ToProtoEnum<PCB_LAYER_ID>`, a visible layer with no protocol
  value), opens one modal dialog per such layer and blocks every later command until a person closes
  them. Outcome per tool:
  - `set_active_layer`: implemented 2026-11-01
    (`docs/development/2026-11-01-07-Set_active_layer.md`). Closed.
  - `set_layer_visibility`: not implemented on KiCad 10.0.7, because reading the visible layers can
    leave KiCad waiting for a person (Rule B); a fix is proposed to KiCad in `docs/upstream/`.
    Closing condition: a KiCad release in which `GetVisibleLayers` no longer asserts.
  - `run_editor_action`: not implemented (decided 2026-11-01, 2 models agree): `RunAction` returns
    only a status, so view actions, the only ones measured safe from dialogs, give an LLM caller
    nothing it can see; the tool name would be permanent public API promising more than a fixed list
    provides; and `editor_navigation.rs` already excludes raw `RunAction` strings by design (#395).
    Closing condition: a tool that needs KiCad to show a result to a person.

- **Sheet files sharing symbol UUIDs** (step 4b): the flags of every sheet file go into one set, so
  if one sheet file is a copy of another under a different name, a flag in one excludes the matching
  part of the other. Closing condition: each part matched only against the file its sheet path
  resolves to, or shared UUIDs reported.
- **Design variants and `exclude_from_sim`** (step 4b): kicad-cli 10.0.7 exports the default variant;
  whether a variant can change a part's `exclude_from_sim` per instance, and so make the flag read
  from the sheet files disagree with the export, is not measured (no demo has variants). Closing
  condition: measured on a schematic with variants.

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
- **~~The IPC paths of this step were not run against KiCad here.~~ Run live 2026-11-01** on stock
  KiCad 10.0.7 with a copy of the pic_programmer demo, through the server binary over stdio:
  `move_footprint_texts` moved R1's reference by 1 mm and the read-back matched; a pad moved off GND
  with `set_pad_net` gave a sync dry run `ready` with `zones_to_refill: ["GND"]` and the
  `zones_need_refill` warning, the apply then `applied`, and after `refill_zones` and `run_drc` with
  `sync_live_board: true` DRC reported 0 errors, 0 unconnected and 0 parity differences (as the
  original demo); `place_component` of `Jumper:SolderJumper-2_P1.3mm_Bridged_Pad1.0x1.5mm` reached
  the saved board with `(net_tie_pad_groups "1, 2")` and `(zone_connect 1)`; a library-refresh dry
  run of that jumper was `noop`, so KiCad's read-back of net ties matches what is written. Not run
  live: `duplicate_component`, `place_array`, an uncertain commit outcome.
- **A stale zone fill after a sync is reported once** (step 7b): the `zones_need_refill` warning is in
  the apply's response only; a later sync returns `noop` and nothing detects a fill left stale, which
  a saved and plotted board then carries. Refilling inside the sync was not chosen (owner ruling
  2026-11-01). Closing condition: a check that reports a fill older than the pads it covers, or a
  recorded decision that the one warning suffices.
- **A pour of a third net under a moved pad is not listed in `zones_to_refill`**, although its
  clearance around the pad can be stale when the old and new nets' netclass clearances differ.
  Closing condition: every zone net whose outline covers a moved pad listed, or this measured as
  harmless on a live board.

## Test isolation and state files (found 2026-11-01)

- **Tests write into the developer's real state directories.** Only
  `crates/copperpour/tests/simulation_live.rs` and `copperpour-sexp/src/writer.rs`'s own tests set
  `COPPERPOUR_STATE_DIR`; the rest of the suite wrote `~/.copperpour/logs/calls.jsonl` (8 MB, through
  `observability.rs`'s `user_state_dir()`), test EasyEDA cache entries, and 107,778 document lock
  files into `~/.local/share/konnect/locks` in one session (deleted by owner ruling, 2026-11-01).
  Same class as the Freerouting entry above. Closing condition: the test harness points
  `COPPERPOUR_STATE_DIR` and the user state directory at a temporary directory for every test.
- **Document lock files are never removed** (`copperpour-sexp/src/writer.rs:588,613-618`): one empty
  file per distinct document path ever written, so the count grows without limit, slowly in normal
  use. Keeping them is deliberate: deleting a lock file another process holds would let the next
  writer lock a new file while the first still holds the old one. Closing condition: a cleanup that
  takes each file's lock before unlinking it (as `run_registry`'s sweep does for `run/` records), or a
  recorded decision to leave it.

## Custom-shape pads (plan step 1, not merged)

- **Footprints with custom-shape pads cannot be placed** (about 3.9% of KiCad's library). KiCad's
  typed IPC pad stack has `PssCustom` with `custom_shapes`
  (`crates/copperpour-ipc/proto/board/board_types.proto:294`). Closing condition: the footprint's
  primitives translated into `custom_shapes`, tested against real library footprints.
