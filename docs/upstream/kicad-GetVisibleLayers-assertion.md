# KiCad report draft: API `GetVisibleLayers` fails an assertion per unnamed layer ID

Prepared 2026-11-01 for KiCad's issue tracker (gitlab.com/kicad/code/kicad). Not filed yet: filing
needs the owner's GitLab account. Patch: `kicad-GetVisibleLayers-enabled-only.patch` (against
master, file `pcbnew/api/api_handler_board.cpp`; in 10.0.7 the same loop is in
`pcbnew/api/api_handler_pcb.cpp`, `API_HANDLER_PCB::handleGetVisibleLayers`).

## Title

IPC API: GetVisibleLayers asserts "Unhandled case in ToProtoEnum<PCB_LAYER_ID>" once per unnamed
layer ID and blocks the API behind modal dialogs

## Description

**KiCad version:** 10.0.7 (Arch Linux package, built with assertions); the same code is on master
(checked 2026-11-01).

**Steps to reproduce**

1. Enable the API server (Preferences → Plugins → Enable KiCad API).
2. Open `demos/pic_programmer/pic_programmer.kicad_pcb` from KiCad's own demos. Its
   `pic_programmer.kicad_prl` has `"visible_layers": "ffffffff_ffffffff_ffffffff_ffffffff"`.
3. Send `kiapi.board.commands.GetVisibleLayers` for that board.

**Result:** an assertion dialog
`api_enums.cpp(377): assert "false" failed in ToProtoEnum<PCB_LAYER_ID, kiapi::board::types::BoardLayer>(): Unhandled case in ToProtoEnum<PCB_LAYER_ID>`,
repeated once per offending ID. Until every dialog is closed by hand, KiCad answers no further API
request, so the client times out on its next command.

**Expected:** the board's visible layers, with no assertion.

**Cause:** the handler converts every bit of `BOARD::GetVisibleLayers()`. A visible set saved as all
ones has all 128 bits set, and 32 of those IDs (64, 66, …, 126) are not `PCB_LAYER_ID` values, so
`ToProtoEnum` reaches its `default:` case (`wxCHECK_MSG`). `LSET::AllLayersMask()` is also all
ones, so it cannot be used to filter.

**Proposed fix:** report only enabled layers, as `handleSetVisibleLayers` already accepts only
enabled ones:

```cpp
LSET visible = board()->GetVisibleLayers() & board()->GetEnabledLayers();

for( PCB_LAYER_ID layer : visible )
    response.add_layers( ToProtoEnum<PCB_LAYER_ID, board::types::BoardLayer>( layer ) );
```

The patch has not been compiled (a KiCad build was not run here); it uses only operations the same
file already uses on `LSET` (`|`, iteration).

## Evidence

Measured with Copperpour's ignored test `stock_editor_command_probe`
(`crates/copperpour-ipc/src/client.rs`, run with `STOCK_PROBE_VISIBLE=1`); the layer IDs without a
name were derived from `include/layer_ids.h` at tag 10.0.7.
