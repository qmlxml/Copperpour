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
Committed as `e1b8c172`.

## Review 2 (robustness) and fix pass 2

Confirmed: setting the environment from the constructor is safe (no constructor in the dependency
tree starts threads, by reading). Fixed:

1. **A test that runs a copy of the test binary from a temporary folder fell back to a fixed name
   in `/tmp`.** `isolate()` now exports the shared directory as `COPPERPOUR_TEST_HOME_ROOT`, and
   `root()` reads it first, so children and copies use the same directory; the fallback is a
   per-process name. Measured afterwards: no `/tmp/copperpour-test-home*` after a gate run or a
   plain `cargo test`.
2. **Under the gate, the real home recorded was the gate's empty home**, so a per-user KiCad was not
   found: the gate exports `COPPERPOUR_TEST_REAL_HOME` itself, which `isolate()` keeps.
3. **The gate emptied the shared directory while another run of the same checkout could be using
   it**: each gate run now creates its own (`mktemp -d` under the build directory), passes it in
   `COPPERPOUR_TEST_HOME_ROOT`, and removes it when the gate ends, however it ends (`trap`).
4. `cargo test --target <triple>` and a custom target directory: `root()` finds the build directory
   by Cargo's `CACHEDIR.TAG`, else by the `deps` folder above the binary. (Measured: Cargo had not
   written `CACHEDIR.TAG` in this checkout's target directory, so the first version, with the tag
   only, sent every plain `cargo test` binary to the `/tmp` fallback; the gate caught nothing because
   it sets the variable. Found while checking finding 1.)
5. `install_homes` (production code reading a test variable): its comment states that the variable
   only adds a home searched after the others, so it changes the choice only when nothing else is
   found.
6. `UNCHANGED` exported in a shell turns isolation off: stated in the crate documentation (the
   gate's empty-home check would show it).
7. The gate's temporary home and per-run directory are removed on every exit, including the
   disk-space stop.
8. The Windows gap is stated more widely: `dirs` ignores `USERPROFILE` and `APPDATA` for
   `home_dir`, `config_dir` and `data_local_dir`.
- **The simulation worker is started with a cleared environment**, so a test-binary worker fell back
  to `/tmp`: in test builds the launcher passes `COPPERPOUR_TEST_HOME_ROOT` through (also in
  `simulation_live.rs`'s own worker launch).
- The new crate's test expected the recorded real home to equal `HOME` at start; under the gate
  that value is supplied, so the test now expects a supplied value to be kept.

Gate (`logs/gate-test-home-fix2.summary`): 3,748 passed, 0 failed, 113 ignored, `home-untouched`
PASS; nothing left in `/tmp` or the build directory afterwards.
Committed as `5224f95f`.

## Review 3 (intent and liveness) and fix pass 3

Confirmed: no test skips more than before (skips depend on system share roots, `KICAD_*`,
`KICAD_DEMOS` and `KICAD_CLI`, none on the home; library lookup falls back to the installation's
own libraries); the only production change is `install_homes` reading `COPPERPOUR_TEST_REAL_HOME`.
Fixed:

1. **A test file without the call was caught only by `scripts/gate.sh`**, which contributors are not
   told to run, and not by CI. New test `every_test_binary_calls_isolate` (in
   `copperpour-test-home`) reads every `tests/*.rs` and the `src/lib.rs`/`src/main.rs` of each crate
   with unit tests and fails, naming the files, under plain `cargo test`. Control: with the call
   removed from `copperpour-render/tests/kicad_svg_test.rs` it fails naming that file.
2. **The gate cannot see that a test skipped** (`cargo test` hides a passing test's output; the gate
   logs before and after hold the same 25 "skip" matches, all test names): predates this change,
   recorded in `docs/BACKLOG.md`.
3. The macOS ngspice search under a per-user KiCad read `HOME`: it uses `install_homes()`.
4. The test-only variables are documented in `DEV.md` (environment variables are public API).
5. The limit for a process started with its environment cleared (no `HOME`, so `dirs` falls back to
   the real home, which the gate's check cannot see) is stated in the crate documentation.
6. `CONTRIBUTING.md` says how to run a test against the real home on purpose
   (`COPPERPOUR_TEST_HOME_UNCHANGED=1`).
7. The Windows `e2e-kicad` workflow has not run with the isolation: recorded in `docs/BACKLOG.md`.

Gate (`logs/gate-test-home-fix3.summary`): 3,749 passed (+1), 0 failed, 113 ignored,
`home-untouched` PASS.
