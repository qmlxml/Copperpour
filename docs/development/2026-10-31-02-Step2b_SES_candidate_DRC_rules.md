# Step 2b — `apply_specctra_ses`: check the candidate with the project's DRC rules

Plan: `2026-10-31-00-Fork_Survey.md`, section 7. Added by owner ruling during step 2 review 3
(`2026-10-31-01-Step2_halthinks_DRC_evidence.md`). This step changes Copperpour's own code; it
is not a contribution from another fork.

## Defect

`apply_specctra_ses` writes the routed board to a new candidate `.kicad_pcb` and ran kicad-cli
DRC on it where it was written. kicad-cli reads rule severities, clearances and net classes from
`<board stem>.kicad_pro` and custom rules from `<board stem>.kicad_dru`. The candidate has its
own stem and no such files, so it was checked with KiCad's defaults. Measured with
`scripts/measure_candidate_drc_rules.sh` on kicad-cli 10.0.7 (project severity
`invalid_outline: ignore`): source board 0 violations, candidate alone 1, candidate with the
project file copied under its stem 0.

## Implementation (commit `5ec098d`)

- New `candidate_drc` (crates/copperpour-core/src/tools/pcb_routing.rs): copies the candidate
  into a temporary directory together with the source's `.kicad_pro` and `.kicad_dru`, renamed to
  the candidate's stem, and runs DRC there. The candidate's directory gains no files; the
  temporary directory is removed afterwards. Diagnostics naming the temporary copy are rewritten
  to name the candidate's directory.
- Schematic parity stays unchecked for the candidate: a root schematic copied alone would lose
  its hierarchical sheets.
- Evidence fields: `rules_source` (`"project"` when a rule file was applied, `"kicad_defaults"`
  when the source project has none) and `rules_applied` (the source files used). The step 2 label
  (`project_rules` as a missing category, `rules_diagnostic`) is removed: the rules are now
  applied, so nothing is missing.
- Tests: `applied_project_rules_are_named_in_the_evidence`,
  `a_source_without_rule_files_is_checked_with_kicads_defaults`,
  `the_candidates_drc_runs_beside_the_source_projects_rule_files` (Unix; a fake kicad-cli
  records what sits beside the board it is given), and the ignored live test
  `candidate_drc_live_applies_the_project_severity`, which passes on kicad-cli 10.0.7 here,
  including its control (the candidate checked alone does report `invalid_outline`).
- Gate (`logs/gate-step2b-impl.summary`): 3,145 passed, 1 failed (the known baseline failure),
  59 ignored (+1: the live test).

## Routing verdict (owner ruling 2026-10-31)

`clean` requires every category and is therefore false for every import. Options put to the
owner: A, a narrower `routing_clean`; B, report parity as not applicable and leave it out of
`clean`; C, no verdict. Both models recommended A; the owner chose A. B was rejected because a
source board that already fails parity would then be reported clean.

Implemented: `routing_clean` is true when design-rule violations and unconnected items were both
checked and are zero, and false when the unconnected-items category is missing.
`routing_clean_excludes` is `["schematic_parity"]`. The tool description, tool directory and
migration note say it is not design acceptance. Tests:
`routing_is_clean_when_rules_and_connectivity_were_checked_and_pass`,
`routing_is_not_clean_when_connectivity_was_not_checked` (states why not acting is correct and
when it would be wrong), `routing_is_not_clean_with_a_violation_or_an_unconnected_item`.

Routing verdict committed as `59733ab`. Gate (`logs/gate-step2b-verdict.summary`): 3,148 passed
(+3 routing tests), 1 failed (the known baseline failure), 59 ignored.

## Review 1 (implementation) and fix pass 1

1. **Project-local footprint libraries were not found by the candidate's DRC** (defect). Moving
   the board into a temporary directory also moved `${KIPRJMOD}` and dropped the project's
   `fp-lib-table`. Measured by the reviewer and reproduced here on kicad-cli 10.0.7: source in
   place, 4 `copper_edge_clearance`; temporary copy, the same plus `lib_footprint_issues`;
   copy with the table copied unchanged, still `lib_footprint_issues` ("not enabled"); copy
   with `${KIPRJMOD}` expanded to the source directory, the same 4 as the source. Fixed:
   `candidate_drc` writes the table with `${KIPRJMOD}` (and the legacy `$(KIPRJMOD)`) expanded
   and lists it in `rules_applied`; `rules_source` stays `"project"` only when a `.kicad_pro`
   or `.kicad_dru` was applied. New fixture `tests/fixtures/candidate_drc_project_lib` (with a
   README recording the measurement) and ignored live test
   `candidate_drc_live_finds_project_local_footprint_libraries`, which passes here. Control:
   with the table copy disabled, the live test fails.
2. **The parity diagnostic described a file in the temporary directory** under the candidate's
   name, and contradicted the earlier advice to place a schematic beside the candidate.
   Replaced with a fixed statement that parity is not checked for an SES candidate and why.
3. **A relative candidate path rewrote diagnostics to the filesystem root.** The rewrite now
   uses the candidate's canonical path. Not covered by a test: a relative path would require
   changing the working directory of the test process, which other tests share.
4. **`<candidate>.drc.json` was still checked for and deleted on rollback**, although DRC no
   longer writes it. Both removed: an unrelated file of that name no longer blocks an import
   and is never deleted by this tool.
5. **The fake-cli test checked file names only.** It now checks the copied contents, that
   `${KIPRJMOD}` was expanded, that the temporary directory was removed, and the exact parity
   diagnostic.

Checked by the reviewer and found correct: `runtime.block_on` of the async `candidate_drc`
inside `spawn_blocking`; the temporary directory is dropped on every early return; ownership
enrichment reads a byte-identical copy; `routing_clean` matches its documentation.

Gate after fix pass 1 (`logs/gate-step2b-fix1.summary`): 3,148 passed, 1 failed (the known
baseline failure), 60 ignored (+1: the library live test).
