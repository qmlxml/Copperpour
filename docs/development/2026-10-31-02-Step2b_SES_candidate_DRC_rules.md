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
Committed as `9f5d843`.

## Review 2 (robustness) and fix pass 2

The reviewer measured on kicad-cli 10.0.7 with copies of `candidate_drc_project_lib`.

1. **A double quote in the project directory** made the expanded table unreadable, giving a
   `lib_footprint_issues` the source does not have, with no warning (measured). KiCad reads a
   backslash literally, so the quote cannot be escaped. The table is now not applied in that
   case, and the new evidence field `library_diagnostic` says why. Spaces, parentheses,
   non-ASCII and backslash sequences were measured to work. Test
   `a_library_table_that_cannot_be_expanded_is_reported_not_applied`.
2. **`$KIPRJMOD` without braces** is accepted by KiCad (measured) and was not replaced. All
   three forms are now replaced. The fake-cli test covers it.
3. **Windows paths**: `canonicalize` gives `\\?\C:\...`, and KiCad URIs use forward slashes
   (inferred; not testable here). The directory is now written through the existing
   `portable_uri` (crates/copperpour-core/src/tools/library.rs, made `pub(crate)`).
4. **Any failure in the DRC setup undoes a correct routing import.** This was Copperpour's
   policy for DRC failures before this step; the step added causes. The avoidable new cause is
   removed: the table is read and expanded as bytes (new `replace_bytes`), so a table that is
   not UTF-8 no longer fails (test `a_library_table_that_is_not_utf8_is_still_applied`). Whether
   a DRC failure should undo the import is a policy question passed to review 3.
5. **kicad-cli was not stopped on timeout**, so it could keep running while the temporary copy
   was deleted. `run_cli_captured` (crates/copperpour-core/src/tools/cli.rs) now sets
   `kill_on_drop`; this applies to every kicad-cli call. Test `a_timed_out_kicad_cli_is_killed`
   (Linux). Control: with `kill_on_drop` removed, it fails. A server killed by SIGKILL still
   leaves its temporary copy behind, as any temporary file would.

Measured and found correct by the reviewer: kicad-cli run on the copy writes nothing into the
source project (it writes `.kicad_prl` beside the copy only); the renamed `.kicad_pro` gives the
same results; `.kicad_dru` without `.kicad_pro` is applied (rules raised a count from 5 to 10),
so `rules_source: "project"` is right; with only `fp-lib-table`, `"kicad_defaults"` with the
table in `rules_applied` is correct because the table carries no rules.

Gate after fix pass 2 (`logs/gate-step2b-fix2.summary`): 3,152 passed (+4), 1 failed (the known
baseline failure), 60 ignored.
Committed as `7c5e5e7`.

## Review 3 (intent and liveness) and fix pass 3

Fixed in this pass:

- **Finding 2: `routing_clean` could be true with nothing routed.** With the project's
  `.kicad_pro` applied, a project severity `unconnected_items: "ignore"` makes kicad-cli report
  an empty unconnected list (measured by the reviewer: 14 unconnected items with default
  severities, `[]` with "ignore"). `candidate_drc` now reads that severity from the source
  `.kicad_pro`; when it is "ignore" the category is reported as missing, with
  `connectivity_diagnostic`, so `routing_clean` is false. Tests
  `an_ignored_unconnected_check_is_recognised_in_the_project_file` and
  `an_ignored_connectivity_check_is_not_evidence_of_routing` (states why not acting is correct
  and when it would be wrong). The same silent zero exists in plain `run_drc` for a project that
  ignores unconnected items; that predates this work and is recorded here as deferred.
- **Finding 5: a library table that was not applied left the evidence complete.**
  `footprint_library` is now a missing category in that case. Test
  `a_library_table_not_applied_makes_the_evidence_incomplete`.
- **Finding 6: `rules_source: "project"` does not prove the rules were parsed** (a `.kicad_dru`
  with a syntax error gives exit 0 and no message, measured by the reviewer; the source board is
  affected in the same way). Documented in the migration note.

Not adopted, after measuring:

- **Finding 4: relative library URIs.** The reviewer reported that `(uri "mylib.pretty")`
  resolves for the board in place and not for the copy. Re-measured here on kicad-cli 10.0.7 in
  the reviewer's own directories: the board in place also reports `lib_footprint_issues` ("The
  footprint library 'mylib' is not enabled"). A first fix that made relative URIs absolute was
  written and then removed, because it made the copy report less than the source. Relative URIs
  are copied unchanged; the live test `candidate_drc_live_finds_project_local_footprint_libraries`
  now also checks that the copy and the source report the same findings with a relative URI.

Put to the owner (see the rulings below):

- **Finding 1:** any failure in the DRC setup or run undoes a routing import that was committed
  and verified, and a failure that repeats (e.g. kicad-cli exceeding its 600 s limit on a large
  board) makes that board impossible to import.
- **Finding 3:** findings already on the source board (e.g. 4 `copper_edge_clearance` on the
  fixture, 17 warnings on KiCad's ecc83-pp demo) are inherited by every candidate, so on such
  boards `routing_clean` can never be true.

Gate after fix pass 3 (`logs/gate-step2b-fix3.summary`): 3,155 passed (+3), 1 failed (the known
baseline failure), 60 ignored.

### Ruling on finding 1 (owner, 2026-10-31): keep the import when only the check failed, and add a re-check

Both models recommended option C; the owner chose it. To implement:
- Roll back only for failures that say something about the import: IPC read-back mismatch and
  candidate write failure. A failure of the DRC setup or run (temporary directory, copy,
  kicad-cli crash or timeout, kicad-cli unable to load the candidate) keeps the import and the
  candidate. The candidate is KiCad's own saved output, verified by read-back, so a load failure
  is treated as a check failure.
- Report such a failure as `drc.status: "not_run"` with the reason, `clean` and `routing_clean`
  false and every category missing, and show the not-run state at the top level of the response
  so a caller reading only `success` sees it.
- The distinction must be typed (which step failed), not inferred from error text.
- Add a way to re-check an existing candidate with the source project's rules (reusing
  `candidate_drc`). The tool description must say that a board on which kicad-cli always
  exceeds its 600 s limit will time out again.
