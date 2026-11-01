# Merge of Copperpour `main` into `integration`

Owner decision 2026-11-01: bring the branch up to date with Copperpour's `main` by a merge, not a
rebase. The branch is published (pull request #1 on `qmlxml/Copperpour`), and a rebase would
rewrite all of its commits and the commit IDs quoted in the step logs.

## What came in

Twelve upstream commits, from the branch's base `f0616624` to `24749e80` (31 October to
1 November): `set_board_stackup` (pull request #232), `set_erc_severity` (#233), the `text_overlap`
check in `check_schematic_overlaps` (#221), and changes to `copperpour init` and `status`: the
`konnect` alias's hooks are replaced, the operating-rules skill is installed as `copperpour`, and
duplicate hooks are reported (#246, #248, #249). A trial merge in a separate worktree found one
conflicting block, in 1 of the 14 files changed on both sides.

## Resolution

- `router/mod.rs`, the list of reviewed open input records: each side had added one
  (`route_specctra_dsn/properties/router_settings` here, `set_erc_severity/properties/severities`
  upstream). Both are kept.
- A conflict git did not report: the first gate run failed `no_toolset_exceeds_max_size`.
  `pcb_board` had 19 tools upstream after `set_board_stackup`; this branch's two unreleased tools
  `list_board_footprint_texts` and `move_footprint_texts` (from the marju212 fork) took it to 21,
  over the limit of 20. `pcb_components`, where they are defined, has 20.
  - Tried first and undone before committing: moving `set_active_layer` to `editor_navigation`.
    That tool is released (it used to always refuse), so moving it would change a published
    toolset.
  - Chosen (the two models agree, each giving its recommendation before seeing the other's): a new
    toolset, `pcb_footprint_text`, holding only the two new tools. No released tool changes
    toolset; the migration note `docs/api-migrations/footprint-text-tools.md` names the new
    toolset. Rejected: putting them in `placement` (that toolset scores and positions footprints
    and does not edit text); moving released tools into a new text toolset (published tools would
    change toolset); raising the limit (the test says to split instead).
  - The toolset count in `DEV.md`, `tool-directory.md` and `packaging/metadata.json` is now 23,
    as `no_file_quotes_a_stale_toolset_count` requires. Per-tool counts are left for the release
    (AGENTS.md).

Merge commit `491a4dfc`. Its message says `set_board_stackup` "took pcb_board to 20 tools"; it took
it to 19 (corrected in fix pass 1). Gate (`logs/gate-merge-main3.summary`): 3,814 passed, 0 failed,
115 ignored, `home-untouched` PASS.

## Review 1 (implementation) and fix pass 1

Confirmed:
- `set_board_stackup` refuses a board open in KiCad (before reading and again after the kicad-cli
  round trip) and writes through the revision-checked replacement, which maps both of this
  branch's conflict errors.
- The client hooks derive their tool lists from the registry, so they cover it and the moved
  tools without change.
- `set_erc_severity` writes only the project file, under the project lock check.
- `text_overlap` only reads, and nothing on this branch uses the count it changes.
- No file changed on both sides merged into duplicated or overwritten code.

Fixed:

1. The PCB skill (`assets/skills/kicad-pcb/SKILL.md`) loaded four toolsets, said the tools it
   mentions are covered by them, and then recommended the two footprint-text tools: it now loads
   `pcb_footprint_text` for silkscreen cleanup, in a `load_toolset(...)` comment that
   `tools_listed_beside_a_toolset_belong_to_it` checks.
2. The operating-rules skill's toolset table (`assets/skills/copperpour/SKILL.md`) lists
   `pcb_footprint_text`. (`editor_navigation` is missing from it too, as it was before the merge.)
3. The `footprint_text_tools` comment and the migration note said `pcb_board` was at the limit of
   20; it has 19.
