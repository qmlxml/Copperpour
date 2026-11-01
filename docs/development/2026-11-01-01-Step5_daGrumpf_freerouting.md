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
