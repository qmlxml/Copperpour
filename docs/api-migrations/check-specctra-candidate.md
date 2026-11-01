### New tool `check_specctra_candidate` (minor release)

Runs KiCad DRC on an existing `apply_specctra_ses` candidate with the source
project's `.kicad_pro`, `.kicad_dru` and `fp-lib-table`, as a temporary copy,
and returns the same `drc` object as `apply_specctra_ses`. Pass
`created_item_ids` from the `apply_specctra_ses` response to get
`import_clean` and the introduced/preexisting split; without them those
fields are `null`. `candidate_sha256` lets the caller confirm the file is the
one the import wrote. A candidate path that names the source board is
refused. Use it when `apply_specctra_ses`
reported `drc_status: "not_run"`. A board on which kicad-cli exceeds its 600 s
limit will time out again.
