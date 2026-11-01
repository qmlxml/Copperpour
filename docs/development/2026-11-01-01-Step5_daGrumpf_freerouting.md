# Step 5 — `daGrumpf-bxp/Konnect`: Freerouting integration and JLCPCB CPL rotations

Plan: `2026-10-31-00-Fork_Survey.md`, section 7. Worked overnight; see
`2026-11-01-00-Overnight_decisions.md` for decisions awaiting the owner.

What the contribution adds (author Pierre Grasswill): `route_specctra_dsn` reports progress as MCP
`notifications/progress`; finds the right Java and checks the Freerouting JAR's Java release;
`optimize_seconds` bounds the optimiser and keeps its best board; job settings sent in the keys
Freerouting reads, with optimiser passes and threshold; `plane_nets` / `plane_as_obstacle`;
`router_settings` passes any Freerouting setting and reports what did not take; a routing that
leaves connections unrouted is `success: false`; non-JSON-RPC output lines skipped; unanswered
polls survived; the log read incrementally; JVM crash reports kept out of the project directory;
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
`java_runtime.rs` and `progress.rs` landed under `copperpour-core`). Two comments that said
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
   complete routing. `success` keeps its meaning (the job finished), but the response now has
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
