# Step 4b: read the schematic's "exclude from simulation" flag

Owner review of overnight decision 1 (2026-11-01; the second reviewer second opinion; 2 models agree): MODIFY.
Since step 4's review 3, a part in KiCad's netlist with no line in its SPICE netlist was listed as
`excluded_in_schematic`, with a warning saying the schematic marks it excluded. That was inferred: a
connector or test point with no model was told the same, which was false.

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
