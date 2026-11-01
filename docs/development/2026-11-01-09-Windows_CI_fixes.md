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

Commit `cb2e9972`; CI on it passes on all ten jobs.

## Review 2 (robustness) and fix pass 2

Confirmed: `names_network_location` cannot panic (byte indexing after a length check) and refuses
every other verbatim form tried (`\\?\C:/…`, `\\?\unc\…`, `\\?\GLOBALROOT\…`, `//?/C:/`); the
creation of the `AppData` folders is safe when test binaries start at once; `KICAD_CONFIG_HOME` on
Windows is still the folder Copperpour reads as KiCad's configuration base; the hung-worker test
cannot give a third result. Fixed:

1. **Mixed leading separators** (`/\host\…`, `\/host\…`) are UNC paths to Windows and passed the
   include check, so the candidate was opened, and an SMB connection made, before the allowed-folder
   check refused it (existed before this step). Any two leading separators now count; tests for both
   forms.
2. **A relative model path under a verbatim folder named no file.** `display_path` keeps the
   `\\?\` form for paths of 260 characters or more, and the include search joins under the
   resolved project folder, which is verbatim on every Windows path. Windows does not treat `/` as a
   separator in a verbatim path or resolve `..` there, so `models/a.lib` was reported missing.
   New `join_relative` (in `flatten.rs`, used by the include search and the snapshot) splits the
   relative part on both separators and applies `.` and `..` under a verbatim folder; any other
   folder is joined as before, so Unix keeps the kernel's handling of `..` through symbolic links.
   Test `a_relative_path_under_a_verbatim_folder_uses_one_separator` (its `..` assertions run on
   Windows only, the one system that parses the prefix).
3. **A timed-out engine probe could leave its working folder behind** on Windows: the child was
   killed without waiting, and the folder, still in use, was removed at once with the error
   ignored. The probe now reads the output itself and, on timeout, kills the worker and waits for it
   before the folder is removed. (The simulation run itself, `run_worker`, was not reviewed for the
   same pattern.)
4. The test-home documentation called the kicad-cli cache a gap; it is now stated as not measured.

Recorded in `docs/BACKLOG.md`, not changed (both existed before this branch, and both change the
security design of the include check): a mapped network drive or a symbolic link to a share is
touched before the allowed-folder check refuses it; a project on a network share cannot simulate
with relative model files, since its model paths are network paths.
