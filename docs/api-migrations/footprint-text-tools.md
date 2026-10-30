### `list_board_footprint_texts` and `move_footprint_texts` (minor release)

- New tools in the `pcb_board` toolset, from the `marju212/Konnect` fork (commit `a8af6d2`).
- `list_board_footprint_texts` lists the texts of placed footprints (reference, value, custom fields,
  plain texts) with layer, absolute and footprint-local position, angle, size and visibility.
- `move_footprint_texts` moves, rotates, re-layers or hides them without moving the footprints. Each
  move gives at most one of `x`/`y` (board mm), `dx`/`dy` (board mm from the current place) or
  `local_x`/`local_y` (footprint-local mm). The batch is checked first and sent as one KiCad edit
  (one undo step); a refused move sends nothing.
- Both need KiCad running with the board open. Nothing existing changed.
