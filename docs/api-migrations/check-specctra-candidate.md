### New tool `check_specctra_candidate` (minor release)

Runs KiCad DRC on an existing `apply_specctra_ses` candidate with the source
project's `.kicad_pro`, `.kicad_dru` and `fp-lib-table`, as a temporary copy,
and returns the same `drc` object as `apply_specctra_ses`. `import_clean` and
the introduced/preexisting split are `null`, because the items the import
created are not known after the fact. Use it when `apply_specctra_ses`
reported `drc_status: "not_run"`. A board on which kicad-cli exceeds its 600 s
limit will time out again.
