# Step 5 — `daGrumpf-bxp/Konnect`: Freerouting integration and JLCPCB CPL rotations

Plan: `2026-10-31-00-Fork_Survey.md`, section 7. Worked overnight; see
`2026-11-01-00-Overnight_decisions.md` for decisions awaiting the owner.

What the contribution adds (author Pierre Grasswill): `route_specctra_dsn` reports progress as MCP
`notifications/progress`; finds the right Java and checks the Freerouting JAR's Java release;
`optimize_seconds` bounds the optimiser and keeps its best board; job settings sent in the keys
Freerouting reads, with optimiser passes and threshold; `plane_nets` / `plane_as_obstacle`;
`router_settings` passes any Freerouting setting and reports what did not take; a routing that
leaves connections unrouted is `success: false`; non-JSON-RPC output lines skipped; a routing no
longer fails on a status poll that gets no answer; the log read incrementally; JVM crash reports kept out of the project directory;
the project's edge and hole rules sent with the job. Also JLCPCB CPL rotation rules backed by
preview observations (`docs/JLCPCB_CPL_CORRECTIONS.md`).

## Merge

18 of the fork's 21 commits cherry-picked with `-x`, author kept. Left out, specific to the
contributor's fork: `e436459` (release 0.13.1 with a `0.13.1-<commit>` release name; Copperpour is at
0.15.0 with its own versioning), `bbceb81` and `9826379` (README: "this is a fork", and removing an
advertisement block).

Conflicts:
- `freerouting_mcp.rs`, Java start-up: Copperpour had since split the command into
  `server_command` and added `JAVA_HEADLESS_ARG` (upstream #927). The contributor's
  `java_runtime::java_executable()` was put into `server_command`; both the headless option and the
  contributor's crash-report options are passed before `-jar`. Both tests kept; Copperpour's
  headless test now checks the options come before `-jar` rather than at fixed positions.
- `tools/integration.rs`, Freerouting availability probe: the contributor's Java release check
  taken, with Copperpour's headless option kept in the version command.
- `lib.rs`: both new modules.
- `docs/JLCPCB_CPL_CORRECTIONS.md`: the contributor's new "Built-in rules" section, with the
  product name kept as Copperpour. The rule id `konnect-jlcpcb-cpl-v1` is data the code reads and
  was kept.
- Count and name files: Copperpour's side (`scripts/keep_head_in_conflicts.py`). Checked afterwards:
  no line the contributor added to those files is missing.

Adaptation: none needed for names or paths (git's rename detection placed every file; the new files
`java_runtime.rs` and `progress.rs` were placed under `copperpour-core`). Two comments that said
"Konnect" now say Copperpour; references to upstream issue #518 keep its name. The workspace
builds; `Cargo.lock` unchanged.

Gate after the merge (`logs/gate-step5-merge.summary`): 3,685 passed (+31), 1 failed (the known
baseline failure), 109 ignored. A first run failed fmt (the merged test was not formatted) and
Copperpour's count test, which read a phrase in the step 4 log as a stale toolset count; both fixed.
Committed as `c9f429c`.

## Review 1 (implementation, focused on the merge) and fix pass 1

The three conflict resolutions were confirmed correct (all JVM options before `-jar` in every
command built). Findings and what was done:

1. **The Freerouting state directory read `HOME` directly** (and accepted a relative
   `XDG_STATE_HOME`), so with a relative `HOME` crash reports went under the project, which the
   contribution meant to prevent. Now uses `app_dirs::home_dir()` and ignores a relative
   `XDG_STATE_HOME`.
2. **On macOS and Windows unrouted connections are never counted**, and the result read as a
   complete routing. (Reversed in fix pass 3.) `success` keeps its meaning (the job finished), but the response now has
   `unrouted_connections_checked`, and `message` says completeness is unknown when it is false.
   The missing log locations are recorded in `docs/BACKLOG.md`.
3. **No migration notes**: added `docs/api-migrations/freerouting-route-specctra-dsn.md` and
   `docs/api-migrations/jlcpcb-cpl-rotations-v2.md`; tool-directory rows for `check_freerouting` and
   `route_specctra_dsn` updated.
4. **The CPL rule set grew from 2 rules to 29 under the same policy id.** Renamed
   `konnect-jlcpcb-cpl-v2` so responses tell the two apart; the migration note says which parts now
   rotate differently. (The step log above said the id "is data the code reads"; the code only
   reports it.)
5. **Progress over HTTP was broadcast to every open GET event stream**, so one client received
   another's progress. The broadcast is removed; HTTP clients get no progress; per-request streams
   recorded in `docs/BACKLOG.md`.
6. **On stdio a progress notification could be written after its response**: a separate writer task
   could still hold it. The task is removed; the main loop writes progress while it waits for the
   call (`tokio::select!`) and drains what is left before writing the response.
7. **A large `optimize_seconds` with no `overall_timeout_seconds` was refused naming
   `overall_timeout_seconds`**, which the caller never sent. Now refused as an invalid
   `optimize_seconds` with the reason.
8. `plane_nets` drops non-string entries; the schema check at dispatch refuses them first (array of
   strings). No change.

Not covered by new tests: 1 (would change environment variables shared by every test in the
process), 2 and 7 (need a Freerouting installation or the full handler), 5 and 6 (need a transport
test with a tool that reports progress). Gate (`logs/gate-step5-fix1.summary`): 3,685 passed,
1 failed (the known baseline failure), 109 ignored.
Committed as `d8460a1`.

## Review 2 (robustness) and fix pass 2

The reviewer could not run Freerouting (the JAR needs Java 25; Java 17 and 21 are installed), so
most findings are from reading; the float and log behaviour were checked with `javap`/`jshell` and
the JAR's own configuration. The new stdio loop was confirmed correct. Fixed:

- **Freerouting's stderr kept only its first 64 KiB**, and the reader then stopped, so the cause of
  a failure on a long run was lost. Now the last 64 KiB are kept and the pipe is read to the end.
- **A valid float setting could be refused**: Freerouting holds some settings as Java `float`
  (measured: `optimizer.improvement_threshold`), and the echo comparison used a tolerance of 1e-9.
  Now 1e-6. A negative number sent as a string (`"-5"`) is now refused like a negative number.
  Tests `a_float_setting_echoed_with_float_precision_is_the_same_value`,
  `a_negative_number_sent_as_a_string_is_refused`.
- **A non-executable `JAVA_HOME/bin/java` was chosen** over a working one on the PATH. Now only an
  executable file is chosen (Unix).
- **CPL bottom-side parts**: every observation behind the rules is of a top-side part, so a
  bottom-side part with a non-zero correction is now listed for review (still corrected), with a
  note to check JLCPCB's preview. **Common-mode chokes** (`L_CommonModeChoke*`) are no longer
  treated as rotation-insensitive. **Angles** are kept in [0, 360) (`normalised_degrees`; no 360.0 or
  -0.0). Tests `angles_are_kept_in_zero_to_360`, `a_common_mode_choke_is_not_rotation_insensitive`;
  `built_in_rules_correct_soic_on_both_sides_and_leave_usb_c_for_review` now expects the bottom-side
  SOIC in the review list.

Recorded in `docs/BACKLOG.md`: a slow snapshot download ending the route, and the final-state race;
stdin not read during a call on stdio (disconnect and cancellation not noticed; predates this work);
the temporary SES left behind on some exits; concurrent calls sharing ports and the log; the
bottom-side CPL formula itself; manifest parsing limits (also below).

Minor, not changed: the JAR manifest is read without a size limit and with case-sensitive attribute
names (the check is then skipped and the run goes ahead); a non-UTF-8 JAR path; reading the whole new
part of the log on each poll.

Gate after fix pass 2 (`logs/gate-step5-fix2.summary`): 3,689 passed (+4), 1 failed (the known
baseline failure), 109 ignored.
Committed as `e097248`.

## Review 3 (intent and liveness) and fix pass 3

No state that cannot be left was found (a stale log gives an unknown count, not a false one, because
lines are filtered by the job's tag). Findings and what was done; decisions are listed in
`2026-11-01-00-Overnight_decisions.md` (items 6, 9, 10):

1. **`success` meant two things**: "fully routed" when the log was read, "the job finished" when not.
   This reverses fix pass 1's choice (overnight decision 6), which broke the rule that a check that
   did not run is never a pass. `success` is now true only when the log shows zero unrouted
   connections and zero clearance violations, the project's rules were found and applied, and the
   settings were read back and verified; otherwise `message` lists each reason. The SES is still
   written.
2. **Clearance violations were ignored**: Freerouting's log reports them ("9 unrouted and 218
   violations"); they are now parsed (`latest_violations`), reported as `clearance_violations`, and
   make `success` false. The description says `success` is not a design-rule verdict.
3. **The project's rules were not a lower limit**: `router_settings` could replace them, or set
   `strict_drc: false` / `autorouter.ignore_net_classes: true`, and still get success. Now refused
   (`relaxations`) unless `allow_below_project_rules: true`, which then never gives `success: true`.
   Test `settings_that_relax_the_project_rules_are_found`.
4. **Zero or several `.kicad_pro` files**: several are now refused (name one with `project_path`); none
   makes `success` false with the reason.
5. **A project rule the JAR did not apply** now makes `success` false and is named in `message`.
6. **An unverifiable settings check looked like a passed one**: new `settings_checked`; false makes
   `success` false.
7. **Array settings were compared by plain equality**, so an echo in another order or with `3.0` for
   `3` was refused every time. Arrays are now compared element by element and lists of names as
   sets. Test `arrays_compare_by_element_and_name_lists_as_sets`.
8. **"Best board" was not explained**: the description now says the board kept with
   `optimize_seconds` is best by the optimiser's score and not DRC-clean or complete by construction.
9. **The bottom-side review warning said "left at KiCad rotation"** for parts that were rotated. It now
   says the part was corrected by the rule, that the rule was observed on top-side parts only, and
   that the other mirror convention differs by 180 degrees.
10. **A wrong top-side rule produced no warning**: every part a built-in rule rotated is now listed in
   the review warnings (designator overrides and project rules are the user's own choice and are
   not). `built_in_rules_correct_soic_on_both_sides_and_leave_usb_c_for_review` updated.
11. HTTP: nothing taken away (progress was new in this step).

Gate (`logs/gate-step5-fix3.summary`): 3,691 passed (+2), 1 failed (the known baseline failure),
109 ignored. Committed as `6675dff`.

## Documentation review (sub-step 16)

Found: an unknown violation count still gave `success: true` (code fixed: it is now a reason for
`false`, like an unknown unrouted count); the tool-directory row still described fix pass 1; a
backlog entry claimed but absent; the CPL migration note said all 29 rules are backed by
observations (24 are, 5 are unverified); the overnight decisions heading; a missing commit hash;
four unclear phrases. All corrected.
