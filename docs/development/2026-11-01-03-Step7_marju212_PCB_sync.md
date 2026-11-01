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
Committed as `b388cc1` (author kept: Marcus Junström).

## `a7886a9`: net ties and footprint zone connection (re-implemented)

Not cherry-picked: its delete_symbol, exclude-from-BOM, field copying and `(unlocked …)` changes are
already in Copperpour, and its `LibraryExtras` duplicated Copperpour's `LibraryFootprintData`. Taken:

- `library_footprint::net_ties` and `zone_connection`, and their test
  `net_ties_and_zone_connection_read_from_a_solder_jumper`, as the fork wrote them. The file values
  0–3 map by name to the API's `ZcsNone`…`ZcsPthThermal` (the API enum numbers differ: 0 is
  `ZcsUnknown`), and `find` reads only the footprint's own clause, not a pad's. New
  `apply_net_ties_and_zone_connection` puts both on a built footprint.
- Sync: `PreparedFootprint` carries both, applied in `apply_library_data`. Test
  `an_added_solder_jumper_carries_its_net_ties_and_zone_connection`.
- Placement: `LibraryFootprintData` gains both, with `read` and `apply_to_item`, used by
  `place_component`, and now also by `place_array` and `duplicate_component`, which before sent the
  bare built footprint with none of the library data. (A first version left the models off a B.Cu
  duplicate; review 1 found that case unreachable, since `duplicate_component` refuses back-side
  sources, and it was removed.) Test
  `placed_items_carry_net_ties_and_the_zone_connection`.
- Library refresh: both clauses accepted; both copied onto the rebuilt footprint; new changed domains
  `net_ties` and `zone_connection`. Without them a refresh that differed only in net ties planned a
  no-op and never sent them. Test `net_ties_and_zone_connection_are_sent_once_then_unchanged` also
  checks that a second refresh is a no-op (`109f890`); control: with the net-tie comparison disabled
  it fails.

Migration note `docs/api-migrations/footprint-net-ties-and-zone-connection.md`.

Gate (`logs/gate-step7-a7886a9.summary`): 3,719 passed (+4), 1 failed (the known baseline
failure), 110 ignored.
Committed as `9c801a7`.

## `04afb36`: not applied

Zone nets are already read by Copperpour (#862, `zone_net`, test `a_zone_pours_only_its_own_net`);
Copperpour ignores a zone on no net, where the fork's reading blocks every change. `delete_trace`
accepting a via UUID would change what an existing public tool accepts; `delete_via` exists and
reports `deleted_type: "via"`. (Overnight decision 16.)

## `7fa1510`: a pad-net change refused only when copper reaches the pad (re-implemented)

The snapshot records each pad's board position and reach (`PadReach`), and the routed copper by net
(`RoutedCopper`: tracks and arc chords as segments, vias as discs, zone nets, an unreadable zone).
The planner refuses when copper of the old net reaches the pad, or a zone pours either net (kept from
Copperpour, #862; `6dad58a` waits for the owner), or a zone's net is unreadable. `snapshot_board`
always reads the copper, so the older rule (any routed net refuses) is now reached only from unit
tests that build a board without it. Differences from the fork:

- **An arc's chords are widened by its sagitta.** The fork used the two chords alone; copper on the
  bulge of an arc then did not count as reaching the pad, which would allow the change. Test
  `an_arc_touches_a_pad_on_its_bulge` puts a pad on the arc 0.59 mm from both chords.
- **A custom-shape pad counts as reached** (its primitives can extend past its anchor size), as does
  a pad whose size cannot be read. Test `a_pad_without_a_bounded_reach_counts_as_touched`.
- A via with no size read is given a 1 mm radius (the fork: 0.5 mm), and the largest layer size is
  used rather than the first.
- Pad positions are taken as board coordinates, as Copperpour's IPC client documents
  (`get_footprint_pads_in`); the proto comment saying "relative to the parent footprint" is out of
  date.
- Geometry is read only from items of their declared type; the routed-net count still reads every
  item, as before.

Tests from the fork, adapted: `a_pad_no_copper_touches_may_leave_a_routed_net` (with the Rule A
statement of when refusing would be right), `copper_reaching_the_pad_still_blocks_the_change`; new:
`with_copper_read_a_pour_or_unreadable_zone_still_blocks`. Migration note
`docs/api-migrations/sync-pad-net-change-by-copper-reach.md`. Not run against KiCad here (backlog).

Gate (`logs/gate-step7-7fa1510.summary`): 3,724 passed (+5), 1 failed (the known baseline
failure), 110 ignored.

Committed as `d31f5fb`.

## Review 1 (implementation) and fix pass 1

Confirmed: `footprint_text.rs` matches the fork's after the rename and depends on nothing of
`a7886a9`; the zone-connection mapping; pads in board coordinates; the sagitta widening covers any
arc; the tests fail if the behaviour regresses. Fixed:

1. **A copper zone on no net was recorded as pouring the empty net**, so every pad gaining or losing
   a net was refused on a board with such a zone. Only named zone nets are recorded now. Served test
   `a_zone_on_no_net_does_not_block_a_pad_gaining_a_net` (R1's VCC pad starts on no net); control:
   with the fix disabled it fails. (A first version of this test moved a pad between two named nets
   and passed with the fix disabled; it was rewritten.)
2. **A routed item whose geometry was not read was ignored by the new rule**, so the change was
   allowed (the old count still saw it, but the planner no longer reads the count when copper is
   read). Such an item's net now goes in `incomplete_nets`, which counts as reaching every pad; a
   missing track width is treated the same way instead of as zero.
3. Pad-stack offset: the IPC messages carry none (checked in the proto), so nothing to read; recorded
   in `docs/BACKLOG.md`.
4. **The "B.Cu duplicate keeps no models" branch was unreachable** (`duplicate_component` refuses
   back-side sources first): removed, with its backlog entry and documentation.
5. A doc comment of `the_schematic_decides_dnp_over_the_library` had moved onto the new test: moved
   back.
6. **The pad-reach tests built the copper by hand**, so nothing tested `snapshot_board`'s reading:
   the served test board now holds tracks, and
   `served_tracks_far_from_or_on_the_pad_decide_the_change` runs a dry run with the real KiCad
   capture of R1 and a VCC track 20 mm away (ready), on the pad (conflict), and with no width
   (conflict); control: with `incomplete_nets` disabled it fails.
7. The step log said a closed board used the old rule; there is no closed-board sync path. Corrected
   above.
8. **Net-tie groups were split on commas only**: now on commas and white space, as KiCad reads them.

Gate (`logs/gate-step7-fix1.summary`): 3,726 passed (+2), 1 failed (the known baseline failure),
110 ignored.
Committed as `325b083`.

## Review 2 (robustness) and fix pass 2

Confirmed: the planner's cost is bounded (once per changed pad, that footprint's pads times the old
net's copper); zero-length tracks; arcs over 180 degrees; no NaN path from IPC; the plan revision is
re-taken at apply. Fixed:

1. **`move_footprint_texts` read a present value of the wrong type as absent** (`"field": 5` moved
   the Reference text and reported success) and ignored misspelt keys. Present values of the wrong
   type, unknown keys, and an entry that changes nothing are refused.
2. **Coordinates had no range check**: `dx` of 1e300 overflowed the nanometre sum (a panic in a debug
   build, a wrapped value in release). Inputs are checked against KiCad's ±2,147 mm, and the result
   with overflow-checked addition.
3. **On a back-side footprint the local frame is mirrored**, which the text tools did not model:
   `local` is now null there and local moves are refused (board-coordinate moves work). Copperpour's
   own footprint graphics tools have the same gap; recorded in `docs/BACKLOG.md`.
4. **The read, check and send were not one commit, and the response was the computed result**: now
   inside `run_commit` (a refused edit or failed send drops the commit), footprints decoded once into
   a map by reference (finding 8), and the response is the texts read back from the board, compared
   with what was sent (angles modulo 360). Served tests in `footprint_text_served_tests` with the
   item-store mock: one `UpdateItems` per batch and the read-back position; a refused edit sends
   nothing; a board that keeps something else is reported. Control: with the read-back refusal
   replaced by success, the last test fails. (A first control disabled only the x comparison and
   the test still passed, because y differed too; it was redone.)
5. **A layer change from front to back left the text unmirrored**: refused.
6. **Review 1's fix 8 was wrong.** KiCad 10.0.7's DRC (`FOOTPRINT::MapPadNumbersToNetTieGroups`)
   splits a net-tie group on unescaped commas only and trims each name; only its IPC writer splits
   on white space too. Splitting on white space would tie pads `1` and `2` for a library whose pad is
   named `1 2`, which could hide a real short. `net_tie_pads` now reads groups as the DRC does
   (commas, `\` escapes, trimming); the difference from the IPC writer is in `docs/BACKLOG.md`.
7. Repeated `net_tie_pad_groups` or `zone_connect` clauses are refused instead of reading the first.
8. `moved` counts distinct texts (two moves of one text: 1).
9. A negative track width counts as unread geometry; a pad with no position or a zero size counts as
   reached.

Gate (`logs/gate-step7-fix2.summary`): 3,730 passed (+4), 1 failed (the known baseline failure),
110 ignored.
