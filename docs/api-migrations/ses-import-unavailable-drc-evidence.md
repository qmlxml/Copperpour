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
candidate. `library_diagnostic` says when the library table could not be
applied (its directory contains a double quote, which cannot be written into
a library URI); footprint-library findings may then differ from the source
board's, `footprint_library` is listed in `missing_categories` and `complete`
is false. When the project sets the unconnected-items check to "ignore",
kicad-cli reports none whatever the routing; that category is then reported
as missing, with `connectivity_diagnostic`, so `routing_clean` is false.
`rules_source: "project"` means the project's rule files were present and
given to kicad-cli; kicad-cli does not report a `.kicad_dru` it cannot parse.

Findings are split into `introduced_violations` (those naming an item the
import created) and `preexisting_violations`. `import_clean` is true when no
finding was introduced and unconnected items were checked and are zero;
`routing_clean` still covers the whole board.

If the candidate's DRC cannot run (kicad-cli failure or timeout, temporary
directory or copy failure), the import is no longer rolled back: it was
already verified by IPC read-back. The response has `drc_status:
"not_run"`, a top-level `warning`, and a `drc` object with `status:
"not_run"`, the `reason`, every verdict false and every category missing.
Rollback still happens for an IPC read-back mismatch or a failure to write
the candidate. Callers that treated an error from this tool as "nothing was
imported" must now check `drc_status`. Violation counts can differ from
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
