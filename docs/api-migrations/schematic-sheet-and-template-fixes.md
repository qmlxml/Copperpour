### Sheet grid snapping, `label_orientation`, hierarchy netlist summary, template wiring (minor release)

- `add_hierarchical_sheet`, `edit_sheet` and `move_sheet` snap a sheet's position and size to the
  1.27 mm grid; the response carries the snapped values. A new position of (50, 50) is written as
  (49.53, 49.53); restating a sheet's current position or size, even off the grid, changes nothing.
  Duplicated sheets are offset 20.32 mm (16 grid steps). A coordinate or size beyond ±100,000 mm is
  refused as an invalid argument naming the field.
- `edit_sheet` refuses a resize that would leave sheet pins past the box's edges, naming the pins,
  or would move a pin onto another pin's point; `edit_sheet_pin` refuses a move onto another pin's
  point and a rename to a name another pin of the sheet already has.
- `connect_to_net` and `batch_connect_to_net` accept `label_orientation` (`"horizontal"`, the
  default, or `"along_stub"`).
- `export_netlist_summary` accepts `schematic_scope` (`"file"`, the default, or `"hierarchy"`); a value
  that is not one of those strings is refused. The hierarchy scope runs kicad-cli, and its response
  has `warnings`: `sheet_files_missing` (referenced sheet files that do not exist, whose parts and
  nets are therefore absent) and `not_the_project_root` (the file given is not the project's root
  schematic and was summarised as if it were).
- `audit_power_rails` judges global power rails on the whole design in a hierarchy review and adds
  `rail_coverage`, `global_rail` and `rail` fields; `(power local)` symbols are not treated as global.
- `apply_template` now wires its connection map instead of returning `connections_to_wire` (removed):
  the response has `connections_made`, `connections_unresolved`, `unconnected_pins` and
  `joined_existing_nets` (template nets the caller did not map in `net_mappings` that already exist in the
  schematic, so the template's parts were joined to them). If wiring fails, the error names the
  parts placed, the placed reference of each template reference, and the nets already written;
  applying the template again would place a second copy.
