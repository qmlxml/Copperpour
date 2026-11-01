### Net ties and footprint zone connection carried from the library (minor release)

From the `marju212/Konnect` fork (behaviour of commit `a7886a9`, re-implemented on Copperpour's code).

- `update_pcb_from_schematic`, `place_component`, `place_array` and `duplicate_component` put a
  library footprint's `(net_tie_pad_groups …)` and footprint-level `(zone_connect …)` on the part they
  place. Before, a solder jumper's bridge reached the board as a short in DRC.
- `place_array` and `duplicate_component` now also carry the library's attributes, description and
  keywords and 3D models, as `place_component` already did (`duplicate_component` refuses a back-side
  source, as before).
- `update_footprints_from_library` accepts both clauses instead of refusing the footprint (KiCad's own
  solder jumpers and test points carry them). Its `changed_domains` can now include `net_ties` and
  `zone_connection`. A library zone connection replaces the board footprint's, as KiCad's refresh
  does, and `preserved.instance_overrides` is then `false`.
- A footprint whose `zone_connect` value is not 0–3 is refused by all of these, naming the value.
