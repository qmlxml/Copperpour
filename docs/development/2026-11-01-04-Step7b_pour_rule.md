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
