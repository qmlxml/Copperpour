# Step 2 — `halthinks/KonnectX`: DRC evidence that was not checked is not reported as zero

Plan: `2026-10-31-00-Fork_Survey.md`, section 7.

## Merge

Cherry-picked onto `integration` with `git cherry-pick -x`, author kept:

| Here | Original | Subject |
|---|---|---|
| `28873e8` | `36eec43` | fix(routing): preserve unavailable SES import DRC evidence |
| `1e3457e` | `44c3935` | fix(drc): withhold silent parity zero when root schematic is absent |

What the two commits change:
- `apply_specctra_ses` used to turn a DRC category that was not checked (`null`) into 0 and
  could then report `"clean": true`. Its `drc` object now has `complete`, nullable counts,
  `missing_categories` and `schematic_parity_diagnostic`; `clean` requires complete evidence.
- `apply_parity_evidence` (used by every tool that runs kicad-cli DRC) used to accept an empty
  parity array as a checked zero unless kicad-cli printed "Failed to fetch schematic netlist for
  parity tests". It now also withholds an empty array when no `<board stem>.kicad_sch` exists.

Conflict: `docs/API_MIGRATIONS.md` only. Copperpour no longer edits that file in pull requests;
each change adds its own note under `docs/api-migrations/`. The contributor's note was moved to
`docs/api-migrations/ses-import-unavailable-drc-evidence.md` and `API_MIGRATIONS.md` kept as
Copperpour has it. The crate rename (`konnect-core` to `copperpour-core`) was applied by git's
rename detection with no conflict.

Gate (`logs/gate-step2-merge.summary`): fmt, clippy, reliability, doc tests, viewer pass; tests
3,140 passed, 1 failed, 58 ignored. Against the baseline: +5 passed and +1 ignored, which are the
contributor's five new unit tests and one ignored live test; the one failure is the known
baseline failure.

## Review 1 (implementation) and fix pass 1

Review 1 found no defect in the logic. Its findings and what was done:

1. **`apply_specctra_ses` can almost never report `clean: true`.** The candidate board must be a
   new file, so its stem differs from the source board's, and KiCad checks parity only against
   `<candidate stem>.kicad_sch`. This is correct behaviour (a parity check that did not run is
   not a pass), but callers were not told. Fixed in documentation: the tool description, the
   `candidate_output_path` schema text, the tool-directory row and the migration note now say
   that parity is normally missing for a candidate and that complete evidence comes from saving
   the board in KiCad and running `run_drc` on the source board.
2. **The `run_drc` behaviour change had no migration note.** Added
   `docs/api-migrations/drc-silent-parity-zero-withheld.md`.
3. **Out-of-date descriptions.** The tool-directory `run_drc` row said the diagnostic quotes
   KiCad; it now says when parity is reported unchecked. The comment on `PARITY_NOT_RUN` said
   KiCad prints the message "whenever" the root is missing; it now records the measurement
   below.
4. **The migration note said the diagnostic is "KiCad's reason".** In the new case it is
   Copperpour's text. Note corrected; the test `unavailable_parity_preserves_the_kicad_diagnostic`
   renamed `unavailable_parity_passes_its_diagnostic_through` and given the real string.
5. **Two live tests might contradict each other.** Measured with the new script
   `scripts/measure_parity_stderr.sh` on kicad-cli 10.0.7 (installed here):
   `board_outline_kicad10.kicad_pcb` alone gives exit 0, an empty parity array and **no
   message**. The contributor's statement is correct, and Copperpour's existing live test
   `live_parity_is_checked_when_the_schematic_is_beside_the_board` would fail on 10.0.7 because
   it required the message. Its assertion now accepts either diagnostic. (That test could not be
   run here: it needs KiCad's demo projects, which the Arch packages do not install. The fixture
   `test.kicad_pcb` is rejected by kicad-cli 10.0.7 with exit 3, so it could not be used for the
   measurement either.)
6. **The contributor's live test cannot be run from this repository** (its fixtures are not
   included) and panicked without explanation on a missing variable. The test now documents
   where its fixtures came from and what to set, and `expect` names the variable. The
   `KONNECT_` prefix is kept: it matches the other live-test variables (`KONNECT_LIVE_*`).
7. **Wording.** "Konnect's side" now names Copperpour; the test
   `a_silent_empty_parity_array_is_a_checked_zero` renamed
   `a_silent_empty_parity_array_with_its_root_present_is_a_checked_zero`.

Gate after fix pass 1 (`logs/gate-step2-fix1.summary`): same as after the merge — 3,140 passed,
1 failed (the known baseline failure), 58 ignored. Committed as `414422f`.

## Review 2 (robustness) and fix pass 2

The reviewer could not run kicad-cli in its sandbox (exit 3 for every case), so KiCad's
behaviour was measured afterwards on this machine with an extended
`scripts/measure_parity_stderr.sh`, using `board_outline_kicad10.kicad_pcb`:

| Case | kicad-cli 10.0.7 result |
|---|---|
| board alone | exit 0, parity `[]`, no message on either stream |
| root `.kicad_sch` empty, unparsable, truncated, unreadable (mode 000), or a directory | exit 255, no report written |

Findings and what was done:

1. **A root schematic that exists but cannot be used would be accepted as a checked zero unless
   KiCad prints its English message** (likely defect, by reading). Measured: on 10.0.7 every such
   root makes kicad-cli exit 255 without a report, so `run_drc` fails instead of returning a zero.
   The state cannot occur on 10.0.7. No logic change; the measurement is recorded in the comment
   on `apply_parity_evidence`. A translated message could not be tested (no other locale is
   installed) and does not matter on 10.0.7, which prints no message for this check.
2. **Only stderr was searched for the message.** `run_drc` now passes stderr and stdout together.
   New test `the_not_run_message_on_stdout_also_withholds_parity` (Unix; a fake kicad-cli prints
   the message on stdout with a root schematic present). Control: with the old stderr-only line
   restored, the test fails; with the fix it passes.
3. **`is_file()` was evaluated twice**, so a file appearing in between gave a diagnostic that
   contradicted itself. Now evaluated once. The remaining case, a root created after kicad-cli
   finished, is recorded in the comment as not closable without KiCad stating what it compared.
4. **Legacy `.sch` roots are not recognised.** Kept: the result is withheld (reported as not
   checked), never invented. Recorded in the comment.
5. **The SES evidence dropped `ownership_diagnostic`.** Now passed through; migration note
   updated; test `an_ownership_diagnostic_is_passed_through` added.

Checked by the reviewer and found consistent: paths with several dots, no extension or an
uppercase extension; relative paths (kicad-cli and the file check resolve against the same
working directory); symlinks; no caller of `run_drc` treats the new `null` as a number. A
non-UTF-8 board path still panics at `pcb.to_str().unwrap()` in `run_drc`; that predates this
change and is not part of this step.

Gate after fix pass 2 (`logs/gate-step2-fix2.summary`): 3,142 passed (the two new tests),
1 failed (the known baseline failure), 58 ignored. Committed as `ff8c657`.

## Review 3 (intent and liveness) and fix pass 3

Review 3 asked whether the intended behaviour is right, not whether the code implements it.

1. **The candidate's DRC runs against KiCad's default rules, not the project's** (defect in the
   specification; Copperpour's existing behaviour, not introduced by the contribution).
   `apply_specctra_ses` writes the candidate with no `<candidate stem>.kicad_pro` or `.kicad_dru`
   beside it. Measured on kicad-cli 10.0.7 with `scripts/measure_candidate_drc_rules.sh` (project
   severity `invalid_outline: ignore`): source board 0 violations; candidate alone 1; candidate
   with the project file copied under its stem 0.
   **Owner ruling 2026-10-31** (both models consulted; the second reviewer recommended this, the first leaned
   to fixing it inside this step and then agreed): label it now, fix it as a separate plan step.
   Done: the `drc` object has `rules_source` (`"project"` or `"kicad_defaults"`); when the source
   project has a `.kicad_pro` or `.kicad_dru` that was not applied, `project_rules` is a missing
   category, `complete` and `clean` are false, and `rules_diagnostic` names the files. When the
   source project has no rule files, the defaults are what the source itself is checked with and
   nothing is marked missing. Limit: the source's rule files are listed only when the candidate
has no `.kicad_pro` of its own; a candidate with a `.kicad_pro` but no `.kicad_dru` is reported
as `"project"` even if the source has custom rules. Step 2b removes this case by copying both
files. Three tests added. The real fix is plan step 2b (Fork Survey
   section 7).
2. **`clean` is always false for this tool, so callers may learn to ignore it.** Not changed in
   this step: reporting parity as "not applicable to an import" would reverse the position taken
   in fix pass 1 (a check that did not run is not a pass). Left to step 2b, which changes how the
   candidate's DRC runs and can propose a routing verdict for the owner to rule on.
3. **The recommended way out did not name Copperpour's tools.** The tool description,
   tool-directory row and migration note now name `run_drc` with `sync_live_board: true` and say
   that it saves the live board over the source file, which `apply_specctra_ses` itself avoids.
4. **A board designed without a schematic can never pass `validate_for_manufacturing` on 10.0.7.** The
   fix text told the user to save a schematic that does not exist. It now also says that such a
   board cannot be shown READY by the audit. A recorded opt-out for board-only projects is not
   added: it is a new feature, recorded here as deferred work.
5. **Putting the candidate beside the source's root schematic** to get parity would break
   hierarchical sheets in a temporary directory, or leave a second project in the user's
   directory. Not pursued.

Gate after fix pass 3 (`logs/gate-step2-fix3.summary`): 3,145 passed (the three new rules
tests), 1 failed (the known baseline failure), 58 ignored. Committed as `3115795`.

## Documentation review (sub-step 16)

An independent check of this log, the plan entry and the migration notes against the
repository found: the manufacturing tool was misnamed (`audit_manufacturing`, which runs no DRC;
the changed text is in `validate_for_manufacturing`); the SES migration note's description of
`complete` predated `project_rules`; the `.kicad_dru` limit above was not stated; the test-count
delta omitted the extra ignored test; the measurement script defaulted to a fixture kicad-cli
10.0.7 rejects and still said the code searches only stderr; and the doc comment on
`apply_parity_evidence` still said the message is always printed to stderr. All corrected.
