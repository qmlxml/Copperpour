### `set_active_layer` changes the active layer over IPC (minor release)

- Before: always refused with `unsupported_capability`, on the claim that the stable KiCad IPC
  protocol had no supported active-layer operation.
- Measured on stock KiCad 10.0.7 (2026-11-01): `GetActiveLayer` and `SetActiveLayer` are handled and
  a change reads back. Now, on a board open in KiCad, the tool sets the layer, reads it back, and
  returns `{active_layer, previous, changed, source: "ipc_readback"}`.
- Still `unsupported_capability`: a board not open in KiCad (the active layer is editor state and
  is never written to the file), and a KiCad that answers the command as unhandled. A layer the
  board does not enable is refused by KiCad (`AS_BAD_REQUEST`), reported as a rejection.
- Arguments unchanged (`board`, `layer`). Behaviour from the wilfredo-a14 fork's `f517fd8`.
