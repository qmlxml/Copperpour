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
  bare built footprint with none of the library data. `duplicate_component` on B.Cu builds on that
  layer instead of flipping, so the models are left off there (backlog). Test
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
Copperpour, #862; `6dad58a` waits for the owner), or a zone's net is unreadable. Without the copper
read (a closed board, the test boards) the old rule applies. Differences from the fork:

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
