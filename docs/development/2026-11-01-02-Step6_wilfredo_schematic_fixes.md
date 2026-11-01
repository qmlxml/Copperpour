# Step 6 — `wilfredo-a14/Konnect` (branch `agent-hub`): schematic and hierarchy fixes

Plan: `2026-10-31-00-Fork_Survey.md`, section 7. Worked overnight; see
`2026-11-01-00-Overnight_decisions.md`.

## Merge

The branch has 15 commits. Merged, cherry-picked with `-x`, author kept (10):
- `4965215` keep sheet pins on the box when `edit_sheet` resizes it; `86ffbaa` move a sheet pin onto
  the edge `edit_sheet_pin` names; `e0dc78d` refuse an `edit_sheet` resize that would strand pins;
  `8c49706` put hierarchical sheets on the 1.27 mm grid; `9900107` tidy sheet geometry left over from
  grid snapping; `fe4bc23` name the dimension a refused sheet resize is short of.
- `8e7ec95` summarise the whole hierarchy in `export_netlist_summary`; `d797298` judge global power
  rails on the whole design (`design_review`); `685b55a` wire the connection map `apply_template`
  describes; `e9de5ed` let stub labels run along vertical stubs (`label_orientation: along_stub`).

Not merged:
- `27ab2a6` (upstream's own fix by Paulius Zaleckas, label UUIDs): already in Copperpour.
- `00f2307` (name the viewer binary this platform looks for): Copperpour had already fixed this,
  more thoroughly (platform suffix, PATH search, never the working directory, #184).
- `c2143d2` (copy custom library fields onto placed symbols): merged, then **reverted**
  (`e97b45f`). Copperpour already carries library custom fields into placed components (`817ea51`,
  closes #3); with both, a field such as LCSC was written twice, Copperpour's placement read-back
  refused the result as ambiguous (`mutation_outcome_uncertain`), and three placement tests failed.
- **`f517fd8` and `d2961f4`, left for the owner** (overnight decision 11): they report every tool
  call to "the fredo fork's API Activity panel", add custom KiCad IPC commands to the protocol
  definitions, post an AI coding client's shell commands to KiCad through a hook, and drive KiCad's editors,
  dialogs, checkers and 3D viewer. They depend on a modified KiCad build that stock KiCad 10 does not
  have; whether this fork should depend on it is a scope decision.

Conflicts: `sch_hierarchy.rs` imports (Copperpour's list plus `snap_to_grid`); the sheet schema
descriptions (the contributor's grid wording, Copperpour's shared `project_name` description); the
duplicate-sheet offset (Copperpour's code, the contributor's 20.32 mm, 16 grid steps);
`schematic.rs` (Copperpour already rounds these coordinates). The viewer commit was skipped.

Adaptation (`17ec356`): three lines still used `konnect_` crate names; `along_stub` had been placed
by the cherry-pick inside an unrelated function (Copperpour had added code where the contributor's
context lines were) and was moved into `impl StubDirection`; the connection-map test adapted to
Copperpour's template fixture.

Gate (`logs/gate-step6-merge.summary`): 3,701 passed (+10), 1 failed (the known baseline failure),
109 ignored.

A process slip, recorded: a first `git revert` was refused because the adaptation edits were not
yet committed, and the `git commit --amend` that followed rewrote the previous commit's message.
The branch was reset to the original commit (`f68faec`) before anything else was committed; the
edits were then committed on their own and the revert redone.
Committed as `5e851e9` (step log), with the merge commits before it.

## Review 1 (implementation, focused on the merge) and fix pass 1

Confirmed correct: `along_stub` in `impl StubDirection` and called by both connect tools; the
duplicate offset; the schema descriptions; the revert left nothing behind; snapped coordinates are
reported back and the read-back compares against what was written. Findings and what was done:

1. **`(power local)` symbols were judged as global rails** in the power-rail audit, so a local
   `+3V3` on one sheet could hide another sheet's missing decoupling. New
   `copperpour_sexp::schematic::local_power_symbol_uuids`; `power_rails` does not count those
   symbols as global. Test `only_power_local_symbols_are_reported_local`.
2. **`apply_template` took the pins it asked to wire as wired**, ignoring the batch call's per-pin
   errors, and a failure on one net did not say which nets were already written. The wired set now
   comes from each batch call's `added` list; its `errors` are reported in `unresolved_connections`;
   the failure names the nets already written.
3. **The merged template test wrote into the real `~/.copperpour/templates`** of the machine running
   the tests (the adaptation commit had removed the contributor's isolation). It now runs through
   Copperpour's `in_isolated_home`, as the other fixture tests do. The two files this session's test
   runs had left there (`one_resistor.json`, `pullups.json`, both test fixtures) were deleted.
4. **The stranded-pin check compared against unrounded sums** (49.53 + 48.26 = 97.78999999999999), so
   a pin exactly on the new edge was refused, and it also ran when the size did not change. The
   edges are rounded as KiCad writes them, and the check runs only on a real resize.
5. **`edit_sheet_pin` with `side` alone could clamp two pins onto one corner**, joining their nets.
   A move onto another pin's point is now refused, naming that pin.
6. **No migration note or tool-directory change** for the caller-visible changes: added
   `docs/api-migrations/schematic-sheet-and-template-fixes.md` and updated eight tool rows
   (`export_netlist_summary` no longer says it never needs kicad-cli).
7. **A malformed `schematic_scope` was read as `"file"`**: now refused. Test
   `a_schematic_scope_that_is_not_a_string_is_refused`.
8, 9. Recorded in `docs/BACKLOG.md`: no response field says an input was snapped, and snapping only
   the size of an off-grid sheet leaves two edges off the grid; and `apply_template`'s wiring finds
   pins by reference, so a `ref_start` colliding with an existing part also wires that part.

Not covered by new tests: 2, 4 and 5 (they need hierarchy or template fixtures that the existing
tests do not set up for these cases).

Gate after fix pass 1 (`logs/gate-step6-fix1.summary`): 3,703 passed (+2), 1 failed (the known
baseline failure), 109 ignored. After the full run, `~/.copperpour/templates` does not exist.
Committed as `dd92bdf`.

## Review 2 (robustness) and fix pass 2

The reviewer measured with kicad-cli 10.0.7 on copies of the `complex_hierarchy` demo. Confirmed
correct: rounding ties away from zero as KiCad does; the pin-collision check at 1e-7 mm; a cyclic
hierarchy and multi-instance sheets; kicad-cli on the live file without kill on timeout;
`(power)`, `(power local)` and empty power values. Fixed:

1. **A coordinate above about 1.8e302 was written as `inf`**, which KiCad cannot load (measured: exit
   3). The sheet tools now refuse `x`, `y`, `width` or `height` beyond ±100,000 mm
   (`coordinate_out_of_range`).
2. **Shrinking a sheet could move a pin onto the corner where a pin of the other edge was**, joining
   their nets. `edit_sheet` now refuses a resize that puts two pins on one point.
3. **The hierarchy summary hid a missing child sheet** (kicad-cli exits 0 with no message; measured):
   new warning `sheet_files_missing`, from a walk of the referenced sheet files.
4. **A child file given to the hierarchy scope was summarised as a root** with no hint (measured): new
   warning `not_the_project_root`. Live test
   `hierarchy_summary_live_reports_missing_sheets_and_a_child_given_as_root` (passes; added to
   `scripts/run_live_tests.sh`).
5. **A wiring call that returned an error, rather than an error result, did not name the nets already
   written**: both paths now do.
6. **`connections_made` was built from the request**: it now lists only the pins connected, with
   `not_connected` beside them.
7. **A pin in two template nets shorted them** (a second label on the same stub): the pin is kept on
   the first net and the conflict reported.
8. **`edit_sheet_pin` could rename a pin to an existing name** (predates this work): now refused.
10. A net is counted as written only if something was added to it.

Recorded in `docs/BACKLOG.md`: the hierarchy scope runs kicad-cli on the live project (it creates a
`.kicad_prl`); parts matched by reference alone; off-grid pin offsets kept on a move. Minor 9 (an
unparsable batch body) is not reachable in practice, since the body is produced in-process.

Gate after fix pass 2 (`logs/gate-step6-fix2.summary`): 3,703 passed, 1 failed (the known baseline
failure), 110 ignored (+1, the live test). A first run failed clippy on the live test's
configuration; fixed.
Committed as `f4c6e3f`.

## Review 3 (intent and liveness) and fix pass 3

Confirmed: no merged commit depends on the two not merged or the reverted one; no sheet can be made
impossible to edit through the tools (a pin can always be moved with `edit_sheet_pin` x+y first).
Findings and what was done:

1. **Restating a sheet's current off-grid position moved it**, and its pins with it, away from the
   parent wires ending on them (19 sheet coordinates in KiCad's demos are on the 0.635 mm half-grid;
   inferred for the vme-wren FPGA sheet). The tests had been changed to on-grid inputs, which hid it.
   `edit_sheet` and `move_sheet` now compare the raw request with the current value (`same_mm`,
   1e-6 mm) before snapping; restating changes nothing, and only new values are snapped (the size the
   same way). Test `restating_an_off_grid_position_moves_nothing`; control: with the comparison
   disabled it fails.
2. Sheet pins are not snapped: recorded in `docs/BACKLOG.md`.
3. **"Pooling counts decoupling across the whole net"**: not adopted. The power-rail audit's "no
   decoupling capacitors" asks whether a rail has any capacitor, a property of the net; whether each
   IC's power pins are decoupled locally is `audit_decoupling`, which checks per pin and per sheet
   and is not pooled. The contributor's tests encode this intent.
4. The pooled verdict does not list its sources: recorded in `docs/BACKLOG.md`.
5. **After a wiring failure the caller could not finish**: the error now names the parts placed, the
   placed reference of each template reference, and the nets already written, and says not to apply
   the template again.
6. **Unmapped template nets joined existing project nets of the same name without a word** (for
   example a template's 6 V `VIN` joining a 24 V `VIN`): new `joined_existing_nets` in the response
   (overnight decision 13).
7. **The documents named `unresolved_connections`; the response uses `connections_unresolved`**:
   the migration note and tool directory corrected.
8. **`next_steps` said to wire every unconnected pin** (unused GPIOs, an NC pin): now "the pins the
   design needs; mark the rest no-connect".
9. **A file that already had two pins on one point was refused on every resize**: only pins the
   resize moved are now checked.

Gate after fix pass 3 (`logs/gate-step6-fix3.summary`): 3,704 passed (+1), 1 failed (the known
baseline failure), 110 ignored.
