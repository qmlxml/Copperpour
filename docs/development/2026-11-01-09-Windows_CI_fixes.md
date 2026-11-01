# Windows CI fixes

The first CI runs of `integration` on the fork (pull request #1 into `qmlxml/Copperpour`'s `main`)
passed every job except `Check & Test (windows-latest)`. Copperpour's own `main` passes that job, so
every failure came from this branch. No Windows build can run on this machine (no MinGW C
compiler for `nng-sys` and `aws-lc-sys`), so each fix is checked by a CI run.

## Run 1 (41326305454): test code did not compile

`tools/simulation.rs` test `the_export_never_writes_into_the_project` used
`std::os::unix::fs::PermissionsExt`. Every fake tool in its module (`with_fake_cli`) is a shell
script with an empty Windows body, so no test in it could run on Windows; the module is now
`#[cfg(unix)]`, as the module before it already was. Commit `70212eb8`. The product, including the
Windows-only live editor (`crates/copperpour/src/live_editor.rs`), compiled.

## Run 2 (41327451982): 18 tests failed

| Group | Tests | Cause | Fix |
|---|---|---|---|
| Include check | 5 in `simulation::flatten`, 5 in `tools::simulation` | The deck check refused any include path starting with `\\` as a network location. `canonicalize` on Windows returns `\\?\C:\…`, which starts with `\\`. | `names_network_location`: a verbatim drive path (`\\?\` + letter + `:\`) is local; plain `\\host`, `//host`, `\\?\UNC\…`, `\\.\…` and `\\?\Volume{…}` stay refused (tests for each) |
| Snapshot | 4 in `simulation::snapshot` | Expectations written with `/`; `Path::join` uses `\` on Windows | Expectations built with the same `join` and escape the code uses |
| Java search order | 2 in `java_runtime` | `/` and `java` written into the expectation | Built with `join` and the platform's binary name; the macOS stub test cannot arise on Windows (`/usr/bin\java.exe` never equals the stub's path) and is limited to other systems |
| Hung worker | `a_hung_worker_times_out_and_is_not_available` | The worker starts with its environment cleared and `PATH` set to the library's folder, so the Windows script's `ping` was not found and the script exited at once (`nonzero_exit`) | A `goto` loop inside `cmd`; the assertion prints the whole status |
| Per-user directories | `a_relative_home_gives_no_relative_user_directory` | Caused by this branch's test isolation (`2026-11-01-08-Test_home_isolation.md`): `isolate()` set `USERPROFILE` to the temporary home, and `dirs` on Windows resolves the roaming folder under it through the known-folder API, which returns nothing when the folder is missing | `APPDATA` and `LOCALAPPDATA` moved to `home/AppData/Roaming` and `home/AppData/Local` (created), where Windows expects them; the assertion prints every directory and variable involved |

The include check was also a product defect, not only a test one. `take_snapshot` resolves the
project folder with `canonicalize` and wrote that form into every relative `Sim.Library` value of
the schematic copy, which kicad-cli exports into the netlist's `.include` lines, so on Windows every
simulation that uses a model file named relative to the project would be refused (found by
reading the code and confirmed by review 1; not run on Windows with a real project). It is fixed in two places: the
snapshot now writes the ordinary form (`C:\…`, through `copperpour_sexp::paths::display_path`; the
resolved form is still used to compare sheet paths), and the check accepts the verbatim form,
which `display_path` keeps for paths of 260 characters or more and for reserved names.

Commit `394a37be`. Run 3 (41329352599): all ten jobs pass, Windows included. The `app_dirs` cause
above was a hypothesis when committed; its passing on Windows confirms it, since the real profile's
roaming folder existed during run 2, so the lookup must have read the temporary `USERPROFILE`.

Gate (`logs/gate-windows-fixes.summary`): 3,750 passed, 0 failed, 113 ignored, `home-untouched`
PASS.

## Review 1 (implementation) and fix pass 1

Confirmed: all 18 failures are addressed; the include target is taken literally (no unescaping), so
the tests' raw strings reach the check unchanged; `project_dir` stays resolved where sheet paths are
compared; on timeout `kill_on_drop` ends `cmd.exe`, and with it the loop (the old `ping` ran as a
separate process); nothing read the old `appdata/roaming` layout. Fixed:

1. The `copperpour-test-home` module documentation still said `dirs` ignores `USERPROFILE` on
   Windows: rewritten from run 3's evidence; whether `home_dir` follows `USERPROFILE` is stated as
   not measured. The backlog entries are updated to match, and the Windows-build entry is closed.
2. The `app_dirs` assertion could not tell a failed known-folder lookup from a relative result: it
   also prints the unfiltered `dirs` values and whether the two `AppData` folders exist.
3. A comment showed `/usr/bin\\java.exe` (the debug escape) for a path with one backslash.

Not changed: the five `tools::simulation` tests passed on Windows in run 3, so their later
assertions, never reached in run 2, hold there too.
