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
Committed as `4012a292`.

## `run_editor_action`: not implemented

Settled between the two models (owner instruction): not implemented. `RunAction` returns only a
status, so a view-only allow-list gives an LLM caller nothing; widening the list needs a live
measurement per action (display toggles are saved in the project settings) and moves toward the
fork's unbounded version. Recorded in `docs/BACKLOG.md`.

## Review 1 (implementation, robustness, intent) and fix pass 1

1. **A failed read-back after an accepted change was reported as a rejection.** The third call's
   failure is now `ipc_outcome_unknown` with `retry_safe: true`, saying the change was accepted.
   Test `a_failed_read_back_after_an_accepted_change_is_safe_to_repeat`; control: with the original
   unclassified read-back it fails.
2. **A KiCad with no PCB editor open, or unreachable, gave an untyped rejection**: these now give
   the typed refusal.
3. **Board access was still `None`**, so client hooks did not give the "open the board in KiCad"
   guidance: now `LiveOnly`, and the catalogue test pins it.
4. The doc comment of `run_action` had moved onto the new method: moved back.
5. **The refusals were worded for file writes** ("could be silently overwritten", "do not repeat the
   mutation") although the active layer is editor state with no undo entry and repeating it is
   safe. The tool now calls `live_board::observe` directly with its own messages; a timeout is
   `ipc_outcome_unknown` with `retry_safe: true`.
6. A layer the board does not enable is `invalid_argument` on `layer`, naming `get_layer_list`.
   Test `a_layer_the_board_does_not_enable_is_an_invalid_argument`.
7. An unknown layer read first gives `previous` and `changed` as `null`.
8. The invalid-name message says to use KiCad's canonical names.
9. The dispatch test uses an address that cannot exist (it had fallen back to `KICAD_API_SOCKET`,
   so on a machine running KiCad it talked to the live editor) and is renamed; new test
   `only_set_unhandled_is_refused_as_unsupported`.

Gate (`logs/gate-set-active-layer-fix1.summary`): 3,743 passed (+3), 1 failed (the known baseline
failure), 113 ignored.
Committed as `c6a6f4e3`.

## Review 2 (robustness) and fix pass 2

Confirmed: retrying is harmless (the board is rebound by path each call; `Recovered` cannot occur
here); the reliability contract's rules are met (read-back, nothing changed on refusal, an accepted
change with a failed read-back reported as accepted). Fixed:

1. **Three board states were all told "KiCad does not hold this board open"**: each refusal now
   says only what KiCad's answer established (no PCB editor at the endpoint; a build that does not
   name its documents, so nothing is claimed; KiCad not reachable; a board this process saw live and
   lost, with the `board_previously_observed_live` code).
2. **The uncertain outcome said both "safe to repeat" and "do not retry the mutation"**: the
   transport's generic instruction is stripped from the cause (`uncertain_cause`).
3. **KiCad busy (`AS_BUSY`) gave an untyped refusal**: it now says nothing was changed and the call
   can be repeated. Test `a_busy_kicad_changed_nothing_and_the_call_can_be_repeated`.
4. **`AS_BAD_REQUEST` to the first read was reported as a bad layer**: only `SetActiveLayer`'s
   answer is mapped to `invalid_argument` on `layer`.
5. The new `board_state: "editor_state_only"` is documented in the migration note and
   `docs/TROUBLESHOOTING.md`.
6. The mock answers `GetOpenDocuments` in KiCad 10's shape (bare file name plus project directory)
   and asserts both commands name that document.
7. The migration note says a successful call records the board as observed live.

Gate (`logs/gate-set-active-layer-fix2.summary`): 3,744 passed (+1), 1 failed (the known baseline
failure), 113 ignored.
