# KiCad report draft: API `GetVisibleLayers` fails an assertion per unnamed layer ID

**Filed 2026-11-01 as KiCad issue #25967**
(https://gitlab.com/kicad/code/kicad/-/work_items/25967), from the owner's GitLab account
(Vasyl Melnichuk, @vasylmeln), with the title and text below and the patch inline; a merge request, if made, is authored
`Vasyl Melnichuk <qmlxml@gmail.com>`, set for that repository only. Patch: `kicad-GetVisibleLayers-enabled-only.patch` (against
master, file `pcbnew/api/api_handler_board.cpp`; in 10.0.7 the same loop is in
`pcbnew/api/api_handler_pcb.cpp`, `API_HANDLER_PCB::handleGetVisibleLayers`).

## Title

IPC API: GetVisibleLayers asserts "Unhandled case in ToProtoEnum<PCB_LAYER_ID>" once per unnamed
layer ID and blocks the API behind modal dialogs

## Description

**KiCad version:** 10.0.7, release build (CachyOS/Arch package); the dialog is wxWidgets' assertion
check (`wxCHECK_MSG`), which this release build keeps. The same code is on master (checked
2026-11-01).

```
Application: kicad-cli x86_64 on x86_64
Version: 10.0.7, release build
Libraries:
	wxWidgets 3.2.12 EGL
Platform: CachyOS, 64 bit, Little endian, wxBase, wayland, KDE, KDE
Build Info:
	wxWidgets: 3.2.11 (wchar_t,wx containers) GTK+ 0.0
	Boost: 1.92.0
	OCC: 7.9.3
	ngspice: 47
	Compiler: GCC 16.2.1 with C++ ABI 1021
Build settings:
	KICAD_IPC_API=ON
```

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

The patch has not been compiled (a KiCad build was not run here). It uses only existing
operations: `operator&( const BASE_SET&, const BASE_SET& )` (`include/base_set.h`), the `LSET`
constructor from `BASE_SET` (`include/lset.h`), and iteration over an `LSET`, as the loop it replaces
already does.

## Evidence

Measured with Copperpour's ignored test `stock_editor_command_probe`
(`crates/copperpour-ipc/src/client.rs`, run with `STOCK_PROBE_VISIBLE=1`); the layer IDs without a
name were derived from `include/layer_ids.h` at tag 10.0.7.
