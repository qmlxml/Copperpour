# Test isolation from the developer's home directory

Backlog item "Tests write into the developer's real state directories" (found 2026-11-01, when one
session's test runs left 107,778 lock files and an 8 MB call log in the owner's home). Design settled
between the two models (owner instruction): first opinion, a per-test-binary startup isolation;
the second reviewer's, the same plus the gate as a detector (option D). 2 models agree.

## Measurement before the change

Each crate's `cargo test --lib --tests` was run with `HOME` pointed at an empty directory
(`RUSTUP_HOME`/`CARGO_HOME` must be pinned: without them rustup downloaded a toolchain, about 1.5 GB,
into each temporary home). What the tests wrote there:

| What | Written by |
|---|---|
| Document lock files, about 2,285 per full run | copperpour-sexp, copperpour-schematic-editor, copperpour-core, schematic-viewer |
| `~/.copperpour/logs/calls.jsonl`, `cache/easyeda` test entries | copperpour-core, copperpour |
| `~/.copperpour/run/<pid>.*`, `.guidance-checked-*` | servers started by copperpour's tests |
| `~/.local/state/freerouting` | copperpour-core |
| KiCad's own `~/.config/kicad/10.0` (`kicad_common.json`, `kicad.json`), `~/.cache/kicad`, `~/.local/share/kicad` | kicad-cli started by tests |

The last row means a plain `cargo test` let kicad-cli write the developer's real KiCad configuration.
(The owner's `kicad_common.json` was found changed at 14:43:42, 50 seconds after the last measurement
run ended and under another session's activity; its API setting was intact.)

## Implementation

- New workspace crate `copperpour-test-home` (`publish = false`): `isolate()` points `HOME`,
  `USERPROFILE`, the four XDG directories, `APPDATA`, `LOCALAPPDATA`, `KICAD_CONFIG_HOME` and
  `COPPERPOUR_STATE_DIR` at `<temp>/copperpour-test-home` (shared by all test processes, so runs do
  not leave one directory each), keeping `RUSTUP_HOME`/`CARGO_HOME` on the real ones. It does nothing
  when `COPPERPOUR_TEST_HOME_UNCHANGED=1`.
- Every test binary (each crate's library tests, copperpour's `main.rs` tests, 29 integration test
  files, the schematic viewer) calls it from `#[ctor::ctor(unsafe)]`, which runs before any test
  thread, so setting the environment is not a race. Children (kicad-cli, spawned servers) inherit it.
  New test-only dependency: `ctor` 1.0.
- Five tests that re-run their own binary with a deliberate `HOME` (relative, unset, or their own
  temporary one) set `COPPERPOUR_TEST_HOME_UNCHANGED=1` on the child; without it the child's own
  isolation overwrote the value under test (`a_relative_home_gives_no_relative_user_directory`
  failed). The one that removes `HOME` only computes search paths.
- `scripts/gate.sh` runs every check with `HOME` set to an empty directory (XDG variables unset,
  `RUSTUP_HOME`/`CARGO_HOME` pinned) and adds a check, `home-untouched`, that fails listing anything
  written there. Its first run found the schematic viewer, a separate project, writing 22 lock files;
  it was isolated too. Doctests and xtask's tests write nothing (measured).
- Test `isolate_points_home_away_and_keeps_the_toolchain` (in a child process).
- `CONTRIBUTING.md` and `AGENTS.md` say every new test binary needs the call.

## Results

- Control: with copperpour-sexp's call removed, its tests wrote 56 files into an empty `HOME`; with
  it, none.
- The machine-dependent failure present since the baseline,
  `a_relative_home_never_reaches_the_working_directory` (an installed KiCad wrote configuration into
  the real home), now passes: the gate passes completely for the first time in this project.
- Gate (`logs/gate-test-home.summary`): 3,747 passed, 0 failed, 113 ignored, `home-untouched` PASS.
- Remaining (in `docs/BACKLOG.md`): CI does not run the empty-`HOME` check yet; on Windows the
  native Specctra bridge directory is not redirected.
