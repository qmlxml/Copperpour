### Net ties and footprint zone connection carried from the library (minor release)

From the `marju212/Konnect` fork (behaviour of commit `a7886a9`, re-implemented on Copperpour's code).

- `update_pcb_from_schematic`, `place_component`, `place_array` and `duplicate_component` put a
  library footprint's `(net_tie_pad_groups …)` and footprint-level `(zone_connect …)` on the part they
  place. Before, a solder jumper's bridge reached the board as a short in DRC.
- `place_array` and `duplicate_component` now also carry the library's attributes, description and
  keywords and 3D models, as `place_component` already did (`duplicate_component` refuses a back-side
  source, as before). A footprint whose library data cannot be read (for example `(attr virtual)`) is
  still placed, without it, and the response carries `library_data_not_applied` with the reason.
- `update_footprints_from_library` accepts both clauses instead of refusing the footprint (KiCad's own
  solder jumpers and test points carry them). Its `changed_domains` can now include `net_ties`. The
  board footprint's zone connection is kept, with its other design-rule overrides (clearance, mask
  and paste margins), as before; when the library's differs, a planned update carries
  `kept_zone_connection` (`board`, `library`). KiCad's Update Footprint takes this whole group from
  the library or keeps it from the board; Copperpour keeps it.
- A footprint whose `zone_connect` value is not 0–3, or with a repeated `zone_connect` or
  `net_tie_pad_groups` clause, is refused by the sync, `place_component` and the refresh, naming the
  problem. Net-tie groups are read as KiCad's DRC reads them: split on commas, names trimmed.
