### `route_specctra_dsn` and `check_freerouting`: Freerouting settings, honest completion, progress (minor release)

`route_specctra_dsn` accepts new arguments: `optimize_seconds` (bounds the optimiser and keeps its
best board; `final_state` `OPTIMIZATION_STOPPED`), `optimizer_max_passes`,
`optimizer_improvement_threshold`, `plane_nets`, `plane_as_obstacle`, `router_settings` (any
Freerouting setting in its JSON shape; a setting Freerouting does not apply, or any negative number,
is refused before routing) and `project_path` (the `.kicad_pro` whose edge and hole rules are sent
with the job; by default the one next to the DSN, else in its parent folder). The project's rules are
now sent by default, so routing results can differ from earlier releases for the same board.

`success` is now true only when Freerouting's log shows no unrouted connections and no clearance
violations, the project's design rules were found and applied, and the settings sent were read back
and verified. Otherwise `success` is false and `message` says why; the SES is still written. New
fields: `unrouted_connections_checked`, `clearance_violations`, `settings_checked`. A routing that
leaves connections unrouted, or whose counts could not be read (Freerouting's log is looked for
where Freerouting keeps it on Linux, or at `FREEROUTING_LOG`), therefore no longer reports success.

`router_settings` may tighten the project's rules but not relax them: a value below the project's
for `copper_to_edge_clearance_um`, `hole_to_hole_um` or `hole_clearance_um`, `strict_drc: false` or
`autorouter.ignore_net_classes: true` is refused unless `allow_below_project_rules: true` is given,
and then `success` is never true. Several `.kicad_pro` files beside the DSN are refused; name one
with `project_path`.

With the stdio transport, a call that carries a progress token receives `notifications/progress`
during routing. The HTTP transport sends no progress.

`check_freerouting` reports `available: true` only when `java -jar <jar> --version` succeeds, and
reports `java_release` / `required_java_release` when the installed Java is too old for the JAR.
