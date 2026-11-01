### `set_active_layer` changes the active layer over IPC (minor release)

- Before: always refused with `unsupported_capability`, on the claim that the stable KiCad IPC
  protocol had no supported active-layer operation.
- Measured on stock KiCad 10.0.7 (2026-11-01): `GetActiveLayer` and `SetActiveLayer` are handled and
  a change reads back. Now, on a board open in KiCad, the tool sets the layer, reads it back, and
  returns `{active_layer, previous, changed, source: "ipc_readback"}`.
- Still `unsupported_capability`: a board not open in KiCad, or KiCad not reachable (the active layer
  is editor state and is never written to the file), and a KiCad that answers the command as
  unhandled. A layer the board does not enable (KiCad answers `AS_BAD_REQUEST`) is
  `invalid_argument` on `layer`. A timeout, or a read-back that fails after KiCad accepted the
  change, is `ipc_outcome_unknown` with `retry_safe: true`: repeating the call is safe. `previous`
  and `changed` are `null` when the layer read first is not a known layer.
- `ipc_outcome_unknown` from this tool carries `board_state: "editor_state_only"`: the uncertain
  part is editor state only, never the design or the file. KiCad busy (`AS_BUSY`) is a refusal that
  says nothing was changed and the call can be repeated.
- A successful call records the board as observed live in KiCad, as other live tools do, so later
  file-fallback tools treat it as possibly held by KiCad.
- The tool's board access is now `LiveOnly`, so client hooks give the "board must be open in
  KiCad" guidance for it.
- Arguments unchanged (`board`, `layer`). Behaviour from the wilfredo-a14 fork's `f517fd8`.
