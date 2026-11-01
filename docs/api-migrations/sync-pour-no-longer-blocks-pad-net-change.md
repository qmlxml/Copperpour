### `update_pcb_from_schematic`: a copper pour no longer blocks a pad-net change (minor release)

Owner ruling 2026-11-01 (plan step 7b), after the `marju212/Konnect` fork's commit `6dad58a`,
re-implemented on Copperpour's code.

- Before: a pad could not change net while a zone poured its old or new net (`routed_pad_net_change`
  conflict), so a part on GND or a poured supply could change net only after the pour was deleted.
- Now the change is planned and applied, as KiCad's Update PCB from Schematic does. The response has
  `zones_to_refill` (the pour nets the reassignments involve) and a `warnings` entry with code
  `zones_need_refill`, in the dry run (what the apply will leave) and after the apply (run
  `refill_zones`, then `save_project`). Until the refill the stored fills are stale, and DRC can
  report shorts or unconnected items there.
- Still refused: a track, arc or via of the pad's old net reaching the pad, and any zone whose net
  cannot be read. A conflicted plan has an empty `zones_to_refill`.
- New response fields: `zones_to_refill` (array of net names) and `warnings` (array of objects with
  `code` and `message`). Callers that relied on the pour refusal must now read `zones_to_refill`.
