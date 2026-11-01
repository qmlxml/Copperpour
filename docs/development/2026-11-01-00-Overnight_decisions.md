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

## Step 5 (daGrumpf-bxp, Freerouting): review 1 (items 5–7), review 2 (item 8), review 3 (items 9–10)

5. **HTTP clients get no progress notifications.** The contribution broadcast them to every open
   event stream, which leaked one client's progress to another; removed rather than rebuilt per
   request. Reverse: restore the `sse_senders` loop in `progress_reporter` (handler.rs).
6. ~~`route_specctra_dsn` keeps `success: true` when Freerouting's log could not be read.~~
   **Reversed by step 5 review 3**: that broke the owner's rule that a check which did not run is
   never reported as a pass. `success` is now false whenever the unrouted or violation count is
   unknown (an unknown violation count was added by the documentation review), a project rule was not applied, the settings were not verified, or no project was found.
   This makes every routing on macOS and Windows report `success: false` until Freerouting's log is
   found there (`docs/BACKLOG.md`).
7. **The CPL policy id changed to `konnect-jlcpcb-cpl-v2`** because the rule set changed. Reverse:
   the `policy_id` in `jlcpcb_cpl_corrections_v1.json`.
8. **Bottom-side CPL parts with a non-zero correction are now listed for review** (still
   corrected). This adds review warnings to existing two-sided boards.
9. **Project design rules are a lower limit for `router_settings`**: relaxing a rule, `strict_drc:
   false` or `autorouter.ignore_net_classes: true` is refused unless `allow_below_project_rules:
   true`. Several `.kicad_pro` files beside the DSN are refused.
10. **Every part a built-in CPL rule rotated is listed in the review warnings** (designator overrides
    and project rules are not).

## Step 6 (wilfredo-a14)

11. **`f517fd8` and `d2961f4` were not merged.** They depend on a modified KiCad build ("the fredo
    fork": an API Activity panel and custom IPC commands) and post an AI coding client's shell commands to
    KiCad. Merge them only if the fork should depend on that KiCad build: cherry-pick both.
12. **`c2143d2` was reverted** as superseded by Copperpour's own custom-field support (it wrote
    fields twice).
13. **`apply_template` reports `joined_existing_nets`** (template nets not mapped by the caller that
    already exist in the schematic) rather than refusing them; the alternative was to require a
    mapping for every such net.
14. **Step 6 review 3's finding that decoupling should not be pooled was not adopted**: the pooled
    check is about the net; local decoupling is `audit_decoupling`, which is not pooled.
15. **Step 7: `6dad58a` (a copper pour no longer blocks a pad-net change; `zones_to_refill`) is
    not applied tonight.** It reverses Copperpour's rule (two tests assert that a pour on the old or
    new net refuses the change). Question for the owner, with a second opinion, at the end of this
    file once the other step 7 work is done. `7fa1510`'s narrower rule (tracks, arcs or vias of
    the old net reaching the pad) is taken, with the pour rule kept.
16. **Step 7: `04afb36` is not applied** (both behaviours already present; `delete_trace` accepting
    via UUIDs would change a public tool's inputs, and `delete_via` exists).

### Prepared for the owner: should a copper pour block a pad-net change? (decision 15)

- **Now (Copperpour):** `update_pcb_from_schematic` refuses to change a pad's net when a zone pours
  its old or new net (tests `a_pour_still_blocks_its_own_net`, `a_pour_blocks_a_pad_moving_onto_its_net`).
  Cost: a part on GND or a poured supply cannot change net until someone deletes the pour.
- **marju212 `6dad58a`:** allow it and list the pour nets in `zones_to_refill`; the caller runs
  `refill_zones`. This is what KiCad's Update PCB does. Cost: the stored fill is wrong until refilled
  (DRC may report a short); verified by the fork on a real board.
- **Third option:** refill inside the same call. Not possible in plan (dry-run) mode, rewrites every
  fill, and can take seconds.
- **Second opinion (second reviewer, asked without a stated preference): option 2**, because a fill is derived
  data that `refill_zones` regenerates, while a track is not. Conditions: report `zones_to_refill` in
  the plan as well as after apply, as a warning naming the next action; collect zone nets separately
  from track nets in `snapshot_board`, so a track still refuses and an unreadable zone still refuses
  (already met since `d31f5fb`: `RoutedCopper` keeps zone nets and unreadable zones apart from track
  geometry, and an unreadable zone refuses); migration note; the tests asserting "not blocked"
  name the states where blocking is right (Rule A).
- **First opinion (this session):** option 2, with the same conditions. 2 models agree.

17. **Step 7 review 3: library refresh keeps the board footprint's zone connection** with its other
    design-rule overrides, and reports a differing library value as `kept_zone_connection`, instead
    of taking it from the library alone (which matched neither of KiCad's two modes). The other
    option was a new argument taking the whole group from the library, as KiCad's "Update clearance
    overrides" does; not added overnight because it is new public API.

### Owner rulings, 2026-11-01 morning

- **Decision 15 (pour blocks a pad-net change): allow it and report the pours to refill**, as
  KiCad's Update PCB does and as fork commit `6dad58a` does. `zones_to_refill` is reported in the dry
  run and after apply, as a warning naming `refill_zones`; tracks, arcs and vias of the old net
  reaching the pad, and an unreadable zone, still refuse. To be implemented as plan step 7b with the
  full cycle.
- **J3Techs native router: assess first, merge nothing (option D).** Build the fork at its own base,
  run its tests, route two or three KiCad demo boards with `route_native`, check each with KiCad's DRC
  and compare completion and violations with Freerouting on the same boards, read the design, and
  report to the owner, who then decides between not taking it and a router-only branch. Second
  opinion (second reviewer) agreed on D first; it would go to the router-only branch directly if D passes.
- **wilfredo `f517fd8` / `d2961f4`: merge nothing now; measure, then a narrow implementation if
  stock KiCad answers.** First opinion (this session) was to leave both out; the second reviewer's was a
  stock-capable subset. The owner asked the two to discuss; the plan they agreed is recorded in
  `docs/BACKLOG.md` ("2 models agree"), and the owner accepted it.
- **"silent zero" added to the forbidden-terms table** of the owner's global instructions file
  (replacement: "an empty result reported as a checked zero when the check did not run"; quoting an
  identifier verbatim is a citation). The one prose use (fork survey, line 72) was rewritten; upstream
  test names and the public migration-note file name are left as they are. 2 models agree.
- **Test leftovers deleted** (`~/.copperpour`, `~/.local/share/konnect/locks` with 107,778 lock
  files; no Copperpour process running) and the two defects recorded in `docs/BACKLOG.md`, "Test
  isolation and state files". 2 models agree.
- **Windows build: checked through GitHub CI** when the owner pushes (pull request from
  `integration` into the fork's `main`), not a local MinGW install. 2 models agree.

### Owner review of the overnight decisions, 2026-11-01 (one at a time, each with a second opinion)

- **1: MODIFY.** Read the schematic's `(exclude_from_sim yes)` flag: flagged parts stay under
  `excluded_in_schematic`; other parts missing from the SPICE netlist get a separate code
  `not_in_spice_netlist` with a true description; both keep the result `modified`. To be done in the
  follow-up work. 2 models agree.
