# Step 4b: read the schematic's "exclude from simulation" flag

Owner review of overnight decision 1 (2026-11-01; the second reviewer second opinion; 2 models agree): MODIFY.
Since step 4's review 3, a part in KiCad's netlist with no line in its SPICE netlist was listed as
`excluded_in_schematic`, with a warning saying the schematic marks it excluded. That was inferred,
not read. (The premise that a part with no model has no line was wrong: review 1 measured that
KiCad 10.0.7 gives such a part a `REF __REF` line; parts with no line were, in four demos, all
excluded by a symbol or sheet flag.)

## Implementation

- **KiCad's netlist does not carry `exclude_from_sim`** (measured on the ecc83 demo with kicad-cli
  10.0.7: it carries `exclude_from_bom` but not the simulation flag). Each netlist part does carry
  its symbol UUIDs (`tstamps`), now kept as `Component::symbol_uuids`.
- `sim_excluded_symbols` reads the snapshot's copies of every sheet file and collects the UUIDs of
  placed symbols marked `(exclude_from_sim yes)`; library symbols are not visited, and a sheet that
  cannot be parsed contributes nothing (its parts are then reported as `not_in_spice_netlist`, still
  `modified`).
- Parts missing from what ran are split: flagged ones stay `excluded_in_schematic`; the rest go to
  the new `not_in_spice_netlist` (field in `circuit`, readiness report and warning, with a true
  description). Both make the result `modified`.
- Tests: `a_component_keeps_its_symbol_uuids`,
  `sim_excluded_symbols_reads_placed_symbols_in_every_sheet`; the existing live test
  `a_part_excluded_in_the_schematic_is_named_and_the_circuit_is_modified` (rectifier demo, R2 flagged)
  passes against kicad-cli 10.0.7 with libngspice, so the UUID match works on real KiCad output
  (`scripts/run_live_tests.sh simulation`: 48 passed).
- Migration note `docs/api-migrations/simulation-toolset.md` and the kicad-simulation skill updated.

The first gate run failed the documentation test on the skill's `not_in_spice_netlist`; it was added
to `NOT_TOOLS` beside `excluded_in_schematic`. Gate (`logs/gate-step4b.summary`): 3,736 passed (+2),
1 failed (the known baseline failure), 110 ignored.

Committed as `bbbe1252`.

## Review 1 (implementation) and fix pass 1

Measured by the reviewer with kicad-cli 10.0.7 on complex_hierarchy, pic_programmer, ecc83 and q17:
a netlist part's own `tstamps` is the placed symbol's `uuid`; a sheet used twice shares symbol
UUIDs between its instances; a multi-unit part has one string per unit, and flagging one unit drops
the whole part; a library symbol's flag and `dnp` have no effect. Fixed:

1. **A sheet instance flagged `exclude_from_sim` was not read**: KiCad dropped its 29 parts and they
   were reported as unexplained. `sim_exclusions` now also reads `(sheet …)` flags, `Component`
   keeps its `sheetpath` UUIDs, and a part counts as excluded when its symbol or a sheet on its path
   is flagged. Live test `the_parts_of_a_sheet_excluded_from_simulation_are_reported_as_excluded`
   (complex_hierarchy, `ampli_ht_vertical` flagged, the horizontal instance not); control: with the
   sheet match disabled it fails.
2. **The `not_in_spice_netlist` reason was wrong** ("usually no model"): now "for a reason the
   schematic does not show"; the migration note, skill and this log corrected.
3. The UUID test uses KiCad's real form, one string per unit.
4. The split is a pure function, `split_missing_parts`, with a unit test
   (`missing_parts_are_split_by_symbol_and_sheet_flags`) on the measured shapes; the rectifier live
   test also asserts that `not_in_spice_netlist` is empty.
5. The `#` reference filter is removed: KiCad's netlist holds no power symbols, and it could only hide
   a part.
6. Design variants are recorded in `docs/BACKLOG.md`.

Gate (`logs/gate-step4b-fix1.summary`): 3,737 passed (+1), 1 failed (the known baseline failure),
111 ignored (+1, the new live test).
Committed as `55b1aef8`.

## Review 2 (robustness) and fix pass 2

Confirmed (measured): the walk sees only the snapshot's sheets and project file (the netlists are
written outside it, and there are no symlinks); KiCad refuses `exclude_from_sim` values other than
`yes`/`no`; an empty `tstamps` falls to `not_in_spice_netlist`, the cautious side. Fixed:

1. **KiCad 10 still honours the older symbol fields** `Sim.Enable` "0" and `Spice_Netlist_Enabled`
   "N" (measured: either drops the part with `exclude_from_sim no`; KiCad's own demos still carry
   `Spice_Netlist_Enabled`). Such a part was reported as unexplained. `excluded_by_flag` now reads
   them. Live test `a_part_disabled_by_the_older_sim_enable_field_is_reported_as_excluded`
   (rectifier, R2); control: with the field check disabled it fails.
2. **A reference two parts share** counted as excluded when one part was flagged: it now counts only
   if every part carrying it is. Unit test extended.
3. Sheet files sharing symbol UUIDs: `docs/BACKLOG.md`.
4. **The flags were read while holding the export lock**, delaying other sessions: they are now read
   before the lock is taken (the lock covers the export and the read of its includes, so it is not
   released earlier).

Gate (`logs/gate-step4b-fix2.summary`): 3,737 passed, 1 failed (the known baseline failure), 112
ignored (+1, the new live test).
