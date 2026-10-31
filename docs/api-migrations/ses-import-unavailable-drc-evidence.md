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

The candidate's DRC now runs with the source project's rules. Previously
kicad-cli checked the candidate with its default severities, clearances and
net classes, because the candidate is a new file with no project file of its
own name. It is now checked as a temporary copy beside the source's
`.kicad_pro` and `.kicad_dru`, renamed to the candidate's stem, and the
project's `fp-lib-table` with `${KIPRJMOD}` replaced by the source project's
directory, so project-local footprint libraries are still found. The
candidate's directory gains no files, and an existing `<candidate>.drc.json`
no longer blocks the import. `rules_source` is `"project"` when a
`.kicad_pro` or `.kicad_dru` was applied and `"kicad_defaults"` when the
source project has neither, and `rules_applied` lists the source files used.
The parity diagnostic now says that parity is not checked for an SES
candidate. Violation counts can differ from
earlier releases for the same board.

`clean` is false for every import, because parity is not checked for the
candidate. `routing_clean` answers what the import changes: it is true when
design-rule violations and unconnected items were both checked and are zero,
and false when the unconnected-items category is missing.
`routing_clean_excludes` is `["schematic_parity"]`. `routing_clean` is not
design acceptance.

`drc.clean` requires complete evidence and zero findings in every category.
Top-level `success` still describes completion of the import, not design
acceptance. An incomplete check does not discard an imported candidate;
restore the matching schematic/project context and rerun verification before
relying on it. Parity is not checked for the candidate: no schematic is
placed beside the temporary copy, because a root schematic copied alone
would lose its hierarchical sheets. It is reported as missing. For complete evidence, run
`run_drc` on the source board with `sync_live_board: true`; it saves the live
board, which now holds the imported routing, over the source file first. Import mutation, candidate persistence, and rollback semantics
are unchanged. Clients must handle nullable counts and require `drc.complete`
before treating zero counts as complete verification evidence.
