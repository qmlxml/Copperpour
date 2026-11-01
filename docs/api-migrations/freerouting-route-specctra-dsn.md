### `route_specctra_dsn` and `check_freerouting`: Freerouting settings, honest completion, progress (minor release)

`route_specctra_dsn` accepts new arguments: `optimize_seconds` (bounds the optimiser and keeps its
best board; `final_state` `OPTIMIZATION_STOPPED`), `optimizer_max_passes`,
`optimizer_improvement_threshold`, `plane_nets`, `plane_as_obstacle`, `router_settings` (any
Freerouting setting in its JSON shape; a setting Freerouting does not apply, or any negative number,
is refused before routing) and `project_path` (the `.kicad_pro` whose edge and hole rules are sent
with the job; by default the one next to the DSN, else in its parent folder). The project's rules are
now sent by default, so routing results can differ from earlier releases for the same board.

A routing that leaves connections unrouted now returns `success: false` with the count in
`unrouted_connections`; the SES is still written. `unrouted_connections_checked` is false when
Freerouting's log could not be read (it is looked for where Freerouting keeps it on Linux, or at
`FREEROUTING_LOG`); the count is then `null` and `message` says completeness is unknown.

With the stdio transport, a call that carries a progress token receives `notifications/progress`
during routing. The HTTP transport sends no progress.

`check_freerouting` reports `available: true` only when `java -jar <jar> --version` succeeds, and
reports `java_release` / `required_java_release` when the installed Java is too old for the JAR.
