# `set_active_layer` over IPC (narrow part of wilfredo-a14 `f517fd8`)

Plan: owner-accepted joint plan for `f517fd8`/`d2961f4` (`docs/BACKLOG.md`), after the stock-KiCad
measurement (`stock_editor_command_probe`, 2026-11-01).

## Implementation

- `copperpour-ipc`: `get_active_layer_in` and `set_active_layer_in`, as the fork wrote them.
- `handle_set_active_layer`: reads the layer, sets it, reads it back; reports `active_layer`,
  `previous`, `changed`, `source: "ipc_readback"`. An `AS_UNHANDLED`/`AS_UNIMPLEMENTED` answer is
  classified inside the IPC call and kept as the typed `unsupported_capability` refusal; a board
  not open in KiCad also stays `unsupported_capability`; a read-back that differs is an error.
- Tests (mock KiCad keeping the active layer): `a_live_board_changes_its_active_layer_and_reads_it_back`,
  `a_kicad_that_does_not_handle_the_command_is_refused_as_unsupported`,
  `a_read_back_that_differs_is_reported`; the earlier refusal test now carries its Rule A statement.
- **Live on stock KiCad 10.0.7** (copy of pic_programmer, through the server binary): B.Cu set and
  read back (previous F.Cu); In5.Cu refused by KiCad (`AS_BAD_REQUEST`, not enabled on a two-layer
  board); F.Cu restored.
- Tool description, tool directory, `docs/TROUBLESHOOTING.md` section on the old
  `(active_layer ...)` token, migration note `docs/api-migrations/set-active-layer-over-ipc.md`.

Gate (`logs/gate-set-active-layer.summary`): 3,740 passed (+3), 1 failed (the known baseline
failure), 113 ignored.
