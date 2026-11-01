# candidate_drc_project_lib

A board whose only footprint comes from a project-local library: `fp-lib-table` names
`mylib` as `${KIPRJMOD}/mylib.pretty`. Used by the ignored live test
`candidate_drc_live_finds_project_local_footprint_libraries` in
`crates/copperpour-core/src/tools/pcb_routing.rs`.

Measured on kicad-cli 10.0.7: the board in place reports 4 `copper_edge_clearance`. A copy in
another directory without the library table also reports `lib_footprint_issues` ("does not
include the footprint library 'mylib'"); with the table copied unchanged it still does ("'mylib'
is not enabled"), because `${KIPRJMOD}` names the copy's directory. With `${KIPRJMOD}` expanded
to this directory, the copy reports the same 4 as the board in place.

The footprint is `PinSocket_1x04_P2.54mm_Vertical_SMD_low_profile_Pin1Left` from KiCad's
`Connector_PinSocket_2.54mm` library. Made by a review agent on 2026-10-31.
