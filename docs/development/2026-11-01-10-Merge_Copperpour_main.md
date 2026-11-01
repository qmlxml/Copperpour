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

Commit `50bd7118`. Gate (`logs/gate-merge-fix1.summary`): 3,814 passed, 0 failed. CI run
41334561437 on the merge commit passes on all ten jobs, Windows included.

## Review 2 (robustness) and fix pass 2

Confirmed:
- `set_erc_severity` refuses an empty map and unknown check names, and handles a project file
  without an `erc` section.
- `text_overlap` cannot panic on NaN coordinates and is fast enough on large sheets (a quadratic
  loop of a few float operations per pair).
- Nothing loads toolsets by a fixed list that would miss `pcb_footprint_text`: the starter kit is
  `project` and `config`, and the hooks are built from the registry.

Fixed (a combination defect): `set_board_stackup` checks its result by re-saving a private copy of
the board with kicad-cli (`resave_board`). That call used the general 600 s limit, and did not use
this branch's option to kill kicad-cli when the limit is reached (`run_cli_owned`'s
`kill_on_timeout`, for calls on a private copy). Upstream has no such option. A stalled kicad-cli
kept the copy open for ten minutes and then went on running; on Windows the copy's temporary
folder could not be removed. `resave_board` now kills kicad-cli on timeout, with a limit of 120 s.
(That did not yet wait for kicad-cli to end before the folder was removed, so on Windows the folder
could still be left behind; completed in fix pass 3.)
Measured: kicad-cli re-saves the largest demo board, `jetson-agx-thor-baseboard` (85 MB), in
6.2 s (KiCad 10.0.7).

Run afterwards, for the first time since the merge: the ignored real-KiCad test
`live_kicad_saves_the_written_stack_unchanged` passes against kicad-cli 10.0.7 (it prints no skip,
and took 2.7 s).

Recorded in `docs/BACKLOG.md` as upstream's own issues, to report there, not changed here:
- the kicad-cli check runs even when the stackup is unchanged;
- `set_erc_severity` reads the project file without the shared lock;
- two approximations in `text_overlap`.

Commit `02904310`. Gate (`logs/gate-merge-fix2.summary`): 3,814 passed, 0 failed.

## Review 3 (intent and liveness) and fix pass 3

Confirmed:
- After a `set_board_stackup` timeout nothing is left that makes later calls refuse. The tool takes
  no lock, and the copy is in a private folder.
- A new toolset for the two footprint-text tools is the right outcome, and is recorded in the form
  `docs/api-migrations/README.md` asks for. The repository has no changelog, and the plugin and
  packaging files list no toolset names.
- `set_erc_severity`'s refusal names the project lock and says what to do.

Fixed:

1. **A timeout read as a fault in the stackup and gave no next step.** It said "could not load and
   save a copy with this stackup", which a caller would take as a reason to change valid values.
   - kicad-cli killed on timeout is now its own error, `CliTimedOut`.
   - `set_board_stackup` reports it as "did not finish ... within 120 s, so the stackup was neither
     checked nor written; nothing is known to be wrong with it. Retry, or set it in KiCad's Board
     Setup > Physical Stackup".
   - Test `a_timed_out_check_is_not_reported_as_a_fault_in_the_stackup`.
2. **After the kill, nothing waited for kicad-cli to end**, so on Windows the copy's folder could
   still be left behind. This was the claim fix pass 2 made without the code to support it.
   - `run_cli_owned` with `kill_on_timeout` now reads the output itself. On timeout it kills
     kicad-cli and waits up to 5 s for it to end before returning.
   - `a_timed_out_kicad_cli_is_killed` now requires the process to be gone when the call returns,
     instead of polling for a second.
   - Control: with the wait replaced by a kill without waiting, it fails ("still exists when the
     call returns").
   - This affects every call on a private copy: the footprint upgrade, `resave_board`, and the
     export that kills on timeout.
3. **The 120 s limit on upstream's tool was not recorded**: `docs/api-migrations/set-board-stackup.md`
   (unreleased) now states it and the refusal.
4. **The live-editor document did not list project-file writers** among the writes still refused
   while KiCad has the project open: it now does, naming `set_erc_severity`.

Not changed:
- `README.md` says "21 on-demand toolsets". It was already wrong before the merge, and its check
  runs only at release (AGENTS.md).
- A configuration key for the 120 s limit: no user has asked for one.
- The Rule A remarks about upstream's own tests are upstream's to make.

Commit `3f54e7d9`. Gate (`logs/gate-merge-fix3.summary`): 3,815 passed, 0 failed, 115 ignored,
`home-untouched` PASS. CI run 41336468988 on it passes on all ten jobs, Windows included (fix pass
2's own run, 41335714410 on `02904310`, passed as well).

## As shipped, against the plan

Planned (owner decision 2026-11-01): merge Copperpour's `main` into `integration`, resolve the one
conflict a trial merge showed, and pass the gate and CI. Shipped as planned, at `24749e80`, plus:
- the new toolset `pcb_footprint_text`, for a conflict the trial merge did not show;
- skill texts updated for it;
- a bounded, killed and waited-for kicad-cli in `set_board_stackup`'s check;
- a timeout reported as such;
- documentation of upstream's tools under this branch's live editor.

Recorded for upstream, not changed here: the issues in `set_board_stackup`, `set_erc_severity`
and `text_overlap` listed in `docs/BACKLOG.md`. Copperpour's `main` keeps moving, so the branch
will need merging again just before a pull request to Copperpour. The pull request description
should then name `pcb_footprint_text` and the 120 s limit as decisions for its maintainers.
