# J3Techs native router: assessment before any merge

Owner ruling 2026-11-01 (option D, 2 models agree): build the fork at its own base, run its tests,
route KiCad demo boards, check each with KiCad's DRC, read the design, and report; nothing merged.
Done by a delegated agent in a separate worktree with its own build directory (both removed after).
Scripts: `scripts/router_assessment/`. [M] = measured, [I] = inferred.

## Build and tests

- konnect-route and konnect-geom: 346 unit tests pass, 0 fail, 1 ignored (74 s) [M].
- The 19 integration tests (13 files, about 7,500 lines) **did not run**: each returns early and is
  reported as passed when `KONNECT_DB1_FIXTURE_DIR` is unset, which points to a private board not in
  the repository [M].
- konnect-core router modules: 99 tests pass, with fake sinks and no KiCad [M].
- Freerouting could not be compared: it needs Java 25 (Java 17 and 21 are installed) [M].

## Method

Copies of three demo projects had all top-level segments, arcs and vias removed, zones kept. Each
was routed by `start_native_route_job` over MCP stdio (release binary, 16 threads, no KiCad IPC, net
classes from the project). The fork has no file-based apply (`apply_route_plan` needs IPC), so
`pcbtool.py apply` wrote each plan's tracks and through vias into a copy of the board; KiCad's DRC
was run with `--refill-zones --severity-all`. Each board was measured with the pour filled (the
normal saved state) and unfilled (zone outlines, no fill), because the fill changed the outcome.

## Results [M]

| Board, input | Connections | Routed | Router's failures | Open after DRC (connections / nets) | Clearance or short errors | Time |
|---|---|---|---|---|---|---|
| Original routed demos | – | – | – | 0 / 0 | 0 | – |
| ecc83-pp, filled | 14 | 13 | congestion 1 | 1 / 1 | 0 | 48 s |
| ecc83-pp, unfilled | 20 | 20 | 0 | 0 / 0 | 0 | 2 s |
| pic_programmer, filled, all classes | 86 | 45 | no_pin 38, unreachable 3 | 41 / 19 | 0 | 184 s |
| pic_programmer, unfilled, all classes | 125 | 71 | no_pin 54 | 37 / 18 | 0 | 2 s |
| interf_u, filled, all classes | 164 | 62 | no_pin 102 | 102 / 78 | 0 | 486 s |
| interf_u, unfilled, all classes | 200 | 89 | no_pin 111 | 100 / 76 | 0 | 2 s |

- **Correct where it routes**: no clearance or short error in any of 11 applied plans; its own open
  count equalled KiCad's on filled inputs; a repeated run gave a byte-identical plan.
- **Completion is low**: 36–62% of connections on simple two-layer through-hole boards. The main
  failure, `no_pin` ("no legal pin where the connection crosses from one tile to another"), looks
  like a structural limit of the tiled detailed router [I].
- **A filled pour of another net blocks its layer**: every filled-input run put no copper on B.Cu
  and no vias, and was 25–500 times slower in the routing stage.
- `zones_are_obstacles`, documented as controlling routing, is read only by the final checker;
  setting it false changed nothing.

## Risks for an LLM caller [M]

- A job with 41–115 connections still open ends `COMPLETED` with `check.violations 0`; the open
  count is only in `stages[route].detail.connections_after`, with no top-level incomplete flag. This
  breaks the owner's rule that work not done is never reported as complete.
- `route_native_region` with nothing routed returned success with no plan while 14 connections were
  still needed.
- By default every non-Default class with tracks of at least 0.5 mm is left unrouted and reported as
  excluded by policy.

## Design and size

Stages: ingest, connectivity, plane fanout, global grid routing, pin access, negotiated tiled detailed
routing, optimizer, removal of failing copper, exact check, plan. The checker uses exact integer
geometry and the optimizer puts back a net whose edit adds violations. No `todo!()` remains; the GPU
path is interfaces only. 59,500 lines, 13,900 of them tests. No router file imports `pcb_components`
or `pcb_sync`. Taking it would add 20 public tools (`native_routing`), a `native-route-runner`
subcommand and job files, plus the crate rename.

## Recommendation of the assessment

Do not take the router at this revision: it completes 36–62% of connections on the simplest demos,
the failure looks structural, and it reports those runs as `COMPLETED`. Look again if the fork fixes
tile-boundary crossings and pour blocking and reports incompleteness at the top level. Its exact
checker could be considered separately.

## Decision (settled between the two models, owner instruction 2026-11-01)

**Not taken at this revision (option A). 2 models agree.** First opinion (this session) and the second reviewer's,
given independently: the router reports runs with up to 115 open connections as `COMPLETED`, and 19
integration tests pass without running, which breaks the owner's rule that work not done is never
reported as complete; and 36–62% completion with a structural cause would not replace Freerouting
even with the reporting fixed. A Freerouting comparison (option D) would not change either point.
Conditions to look again: tile-boundary crossings and pour blocking fixed, incompleteness reported
at the top level, and the measurement repeated with `scripts/router_assessment/` on one of the
owner's own boards as well as the demos. Its exact checker is to be assessed alone only if it can be
separated from the router and fills a gap Copperpour has.
