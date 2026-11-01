### DRC withholds an empty schematic-parity result when the board has no root schematic (minor release)

`run_drc`, `get_drc_violations` and every tool that runs KiCad DRC used to
report `schematic_parity` as a checked zero when kicad-cli wrote an empty
parity array without printing "Failed to fetch schematic netlist for parity
tests". kicad-cli 10.0.7 does exactly that for a board with no
`<board stem>.kicad_sch` beside it. Such a result is now `null`, with a
`schematic_parity_diagnostic` that names the expected root schematic, and
`schematic_parity` is listed among the missing categories. Manufacturing
readiness now adds an error issue for such a board instead of passing it.

Callers that treated a zero parity count as a pass must now handle `null`.
To get a checked parity result, keep the board beside its project's root
schematic of the same name.
