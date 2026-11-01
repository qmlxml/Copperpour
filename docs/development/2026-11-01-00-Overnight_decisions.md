# Decisions taken overnight, awaiting the owner's review — 2026-11-01

The owner asked for the work to continue overnight. Decisions that would normally be put to the
owner were taken by the assistant, choosing the option that reports more and is easy to reverse. Each is
listed here with where it is recorded, so it can be confirmed or changed. Nothing was pushed.

## Step 4 (yuvaltal912-sys, SPICE simulation), review 3

1. **Parts the schematic excludes from simulation now make the result `modified`** and are listed
   (`circuit.excluded_in_schematic`, warning `excluded_in_schematic`, readiness field). Before, such
   a part vanished and the result read `as_drawn` (measured on the rectifier demo). This changes
   the meaning of `simulated_circuit` for those schematics. Reverse: drop the new condition in
   `circuit()` in crates/copperpour-core/src/tools/simulation.rs.
2. **New warnings** `testbench_elements_added`, `control_block_dropped`, `schematic_open_in_editor`.
   A dropped `.tran`/`.ac` (the schematic's own analysis, always replaced by the requested one) is
   deliberately not warned about, to keep warnings meaningful.
3. **Skill wording**: the I²t fuse stand-in is said not to be evidence that a real fuse protects the
   circuit; approximations on protection or inrush parts are said not to be evidence about them.
4. **The engine-not-found message** names the configuration key and that a server restart is needed (`reload_server` also works with the stdio transport on Unix).

Details: `2026-10-31-04-Step4_yuvaltal912_simulation.md`, "Review 3".

## Step 5 (daGrumpf-bxp, Freerouting), review 1

5. **HTTP clients get no progress notifications.** The contribution broadcast them to every open
   event stream, which leaked one client's progress to another; removed rather than rebuilt per
   request. Reverse: restore the `sse_senders` loop in `progress_reporter` (handler.rs).
6. **`route_specctra_dsn` keeps `success: true` when Freerouting's log could not be read**, with the
   new `unrouted_connections_checked: false` and a message that completeness is unknown. The
   alternative was to report such runs as failures.
7. **The CPL policy id changed to `konnect-jlcpcb-cpl-v2`** because the rule set changed. Reverse:
   the `policy_id` in `jlcpcb_cpl_corrections_v1.json`.
