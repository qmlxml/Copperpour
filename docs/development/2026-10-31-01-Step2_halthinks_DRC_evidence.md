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
