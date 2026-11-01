# Step 7b: a copper pour no longer blocks a pad-net change

Owner ruling 2026-11-01 (overnight decision 15; second opinion from a second reviewer; 2 models agree): allow the
change and report the pours to refill, as KiCad's Update PCB does and as the `marju212` fork's
`6dad58a` does. Base: `integration` at `61e97e13` (gate 3,730 passed after step 7).

## Implementation

- `plan_sync`: the two refusals for a pour on the old or new net are replaced by recording the poured
  nets in `SyncPlan::zones_to_refill`, when no copper of the old net reaches the pad. Copper reaching
  the pad and an unreadable zone still refuse, with their causes named (step 7, review 3). A
  conflicted plan's `zones_to_refill` is emptied, since it changes nothing.
- `SyncResponse`: new `zones_to_refill` and `warnings`; `refill_warning` words the
  `zones_need_refill` warning for the dry run and for after the apply.
- Tests: `a_pour_still_blocks_its_own_net` and `a_pour_blocks_a_pad_moving_onto_its_net` replaced
  by `a_pad_leaving_a_poured_net_asks_for_a_refill` and
  `a_pad_moving_onto_a_poured_net_asks_for_a_refill` (the second also applies and checks the
  after-apply warning); `with_copper_read_a_pour_or_unreadable_zone_still_blocks` replaced by
  `with_copper_read_a_pour_asks_for_a_refill_and_an_unreadable_zone_blocks`; new
  `a_poured_net_with_copper_on_the_pad_still_blocks`. Each states, per Rule A, why not refusing is
  right and when refusing would be. The three old tests failed against the new code before being
  replaced, which is the control.
- Tool description, tool directory row, migration note
  `docs/api-migrations/sync-pour-no-longer-blocks-pad-net-change.md`.

Gate (`logs/gate-step7b.summary`): 3,731 passed (+1), 1 failed (the known baseline failure), 110
ignored.
Committed as `922977f9`.

## Review 1 and fix pass 1

The review covered the implementation, robustness and intent viewpoints in one pass; reviews 2 and
3 are run separately as well, as the workflow requires. Confirmed: the apply's response reports the
plan it applied (re-planned from a fresh snapshot); `zones_to_refill` is deduplicated, records both
nets of a pad moving between two pours, and is kept whole in `output` files; the plan revision need
not hash it (it is derived from the zones, whose count the revision hashes). Fixed:

1. **A plan refused after the pad loop** (preflight failure, stale revision) still carried
   `zones_to_refill` and a warning saying the apply would leave fills stale. The response now names
   no zone and no warning whenever its status is `conflict`. Test
   `a_conflicted_response_names_no_zone_to_refill`.
3. A test's Rule A statement still said refusing was right when a zone poured VCC, and cited two
   removed tests: rewritten. The tool directory row now also names the unreadable-zone refusal.
5. **An uncertain commit outcome dropped the refill**: the plan's pour nets are kept outside the IPC
   closure, and the uncertain result gains "If the commit took effect: …" with the refill warning.
6. The warning after the apply also names `run_drc` with `refill_zones: true`, says a saved and
   plotted board can carry the short, and that it is reported only in this response. Refilling
   inside the sync was not chosen by the owner; the one-time report is in `docs/BACKLOG.md`.
7. The Rule A statement says the rule depends on a live KiCad being present, so a future file-only
   path does not inherit it.
8. A pour of a third net under a moved pad is not listed: `docs/BACKLOG.md`.

Gate (`logs/gate-step7b-fix1.summary`): 3,732 passed (+1), 1 failed (the known baseline failure),
110 ignored.
