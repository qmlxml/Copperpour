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
  `COPPERPOUR_STATE_DIR` at a directory shared by all test processes (first `<temp>/copperpour-test-home`;
  moved to `<target>/copperpour-test-home` in fix pass 1), keeping `RUSTUP_HOME`/`CARGO_HOME` on the real ones. It does nothing
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

Committed as `274ba449`.

## Review 1 (implementation, robustness, intent) and fix pass 1

Confirmed: every test binary is covered (29 integration files, each library's unit tests,
`copperpour`'s `main.rs`, the viewer); `src/bin/konnect.rs` has no tests. Fixed:

1. **A KiCad installed for one user was no longer found** (user flatpak, `~/Applications/KiCad`):
   kicad-cli tests would print SKIP and pass without running. `isolate()` records the real home in
   `COPPERPOUR_TEST_REAL_HOME`, and `kicad_install::install_homes` searches it as well as `HOME`.
   Test `a_per_user_installation_is_searched_in_the_real_home_too`.
2. **The macOS simulation live test read KiCad's model cache from the temporary `HOME`**: it reads
   the real home from that variable.
3. **Copperpour and kicad-cli read different KiCad configurations in tests**: `KICAD_CONFIG_HOME` had
   its own folder, while Copperpour reads the global tables from `<HOME>/.config/kicad`. Measured:
   KiCad appends its version folder (`10.0`) to `KICAD_CONFIG_HOME`, so it is now set to the same
   `kicad` base Copperpour reads, per platform.
4. **A fixed folder in the shared `/tmp`** could be created first by another user, even as a link
   into this user's home: the shared directory is now `<target>/copperpour-test-home`, found from the
   test binary's path (per user and checkout; `cargo clean` removes it), with a per-user temporary
   folder as fallback.
5. **Runs could depend on earlier runs** (KiCad configuration, Freerouting state, ever-growing lock
   files): the gate empties the shared directory before testing.
6. The toolchain variables are pinned only from an absolute `HOME`.
7. `KICAD_DOCUMENTS_HOME` is set too, for kicad-cli on Windows; the remaining Windows gaps are
   stated in the crate's documentation.
8. `gate.sh` stops cleanly when `HOME` is unset or the temporary home cannot be created.
9. CI does not run the empty-`HOME` check: kept in `docs/BACKLOG.md` (it can be added and watched
   only once the branch is pushed).

Gate (`logs/gate-test-home-fix1.summary`): 3,748 passed, 0 failed, 113 ignored, `home-untouched`
PASS.
