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
