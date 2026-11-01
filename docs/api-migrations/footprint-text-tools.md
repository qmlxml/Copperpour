### `list_board_footprint_texts` and `move_footprint_texts` (minor release)

- New tools in a new toolset, `pcb_footprint_text` (the limit is 20 tools per toolset;
  `pcb_components` has 20 and `pcb_board` 19), from the `marju212/Konnect` fork (commit `a8af6d2`).
- `list_board_footprint_texts` lists the texts of placed footprints (reference, value, custom fields,
  plain texts) with layer, absolute and footprint-local position, angle, size and visibility.
- `move_footprint_texts` moves, rotates, re-layers or hides them without moving the footprints. Each
  move gives at most one of `x`/`y` (board mm), `dx`/`dy` (board mm from the current place) or
  `local_x`/`local_y` (footprint-local mm). The batch is checked first and sent as one KiCad edit
  (one undo step); a refused move sends nothing.
- On a back-side footprint, `local` is `null` in `list_board_footprint_texts`, and
  `local_x`/`local_y` are refused: the mirrored local frame there has not been measured.
- `move_footprint_texts` refuses an unknown key, a present value of the wrong type, an entry that
  changes nothing, a coordinate outside KiCad's ±2,147 mm, and a `layer` on the other side of the
  board (the text would need mirroring). The batch is read, checked and sent inside one KiCad
  commit; the response's `texts` are read back from the board and `moved` counts distinct texts. If
  the read-back fails, or the board holds something other than what was sent, the result is an
  uncertain outcome after a mutation (`IpcOutcomeUnknown`), naming each difference; the batch is one
  undo step in KiCad.
- Both need KiCad running with the board open. Nothing existing changed.
