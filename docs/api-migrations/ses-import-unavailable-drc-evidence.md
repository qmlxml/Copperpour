### `apply_specctra_ses` keeps unavailable DRC checks distinct from zero findings (minor release)

`apply_specctra_ses` no longer reports unavailable unconnected-item or
schematic-parity checks as zero findings. In its `drc` object, unavailable
category counts are now `null`; checked counts remain integers. The new
`complete` flag is true only when both categories were reported, and
`missing_categories` names those that were unavailable. The
`schematic_parity_diagnostic` field preserves KiCad's reason when parity
could not run, such as a missing candidate root schematic.

`drc.clean` requires complete evidence and zero findings in every category.
Top-level `success` still describes completion of the import, not design
acceptance. An incomplete check does not discard an imported candidate;
restore the matching schematic/project context and rerun verification before
relying on it. Import mutation, candidate persistence, and rollback semantics
are unchanged. Clients must handle nullable counts and require `drc.complete`
before treating zero counts as complete verification evidence.
