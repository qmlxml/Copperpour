### `apply_specctra_ses` keeps unavailable DRC checks distinct from zero findings (minor release)

`apply_specctra_ses` no longer reports unavailable unconnected-item or
schematic-parity checks as zero findings. In its `drc` object, unavailable
category counts are now `null`; checked counts remain integers. The new
`complete` flag is true only when every category was checked, and
`missing_categories` names the rest. The
`schematic_parity_diagnostic` field says why parity could not run: KiCad's
own message when it printed one, otherwise Copperpour's statement that the
expected root schematic is missing. `ownership_diagnostic` says when
Copperpour could not name what owns each violation item.

`rules_source` says which DRC rules kicad-cli applied to the candidate:
`"project"` when `<candidate stem>.kicad_pro` exists beside it, otherwise
`"kicad_defaults"`. When the candidate has no project file and the source
project has a `.kicad_pro` or `.kicad_dru`, `project_rules` is listed in `missing_categories`,
`complete` is false, and `rules_diagnostic` names the files: the violation
count was computed with KiCad's default severities, clearances and net
classes, not the project's.

`drc.clean` requires complete evidence and zero findings in every category.
Top-level `success` still describes completion of the import, not design
acceptance. An incomplete check does not discard an imported candidate;
restore the matching schematic/project context and rerun verification before
relying on it: KiCad checks the candidate's parity only against
`<candidate stem>.kicad_sch` beside it, which normally does not exist, so
parity is normally reported as missing. For complete evidence, run
`run_drc` on the source board with `sync_live_board: true`; it saves the live
board, which now holds the imported routing, over the source file first. Import mutation, candidate persistence, and rollback semantics
are unchanged. Clients must handle nullable counts and require `drc.complete`
before treating zero counts as complete verification evidence.
