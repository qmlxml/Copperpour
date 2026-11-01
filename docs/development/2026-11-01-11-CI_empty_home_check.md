# The empty-home check in CI

Backlog (test isolation, `2026-11-01-08-Test_home_isolation.md`): "CI does not run the empty-`HOME`
check itself", deferred until the branch could be pushed and CI watched. That became possible with
pull request #1 on `qmlxml/Copperpour`.

## Implementation

Commit `e214f942` changed the `check` job of `.github/workflows/ci.yml`, on Linux and macOS only:
- a step before the tests points `HOME` at an empty directory under the runner's temporary folder;
- `CARGO_HOME` and `RUSTUP_HOME` stay on the real home, as in `scripts/gate.sh` (without them
  rustup downloads a toolchain into the empty home);
- a step after the doctests fails if anything appears in the empty home.

It is left out on Windows, where the per-user folders come from the user profile through the
known-folder API, not from `HOME`.

Checked locally by running both steps' shell, taken from the parsed YAML: an empty home passes,
and a written file (control) makes the check fail, naming the file. CI run 41342827418 passed on
all ten jobs. Its log shows the test step's environment with `HOME` at the empty directory and
`CARGO_HOME` and `RUSTUP_HOME` on the real home, on both systems, and "nothing written" from the
check on both.

## Review (implementation, robustness, intent) and fix pass

This change was reviewed by one reviewer taking the three stances in turn, not by three separate
reviews.

Fixed:
1. **The failing result had been observed only locally, with GNU `find`.** The check is now a
   script, `scripts/ci_home_untouched.sh`. Its `selftest` mode runs on every CI build before the
   tests. It checks that a written file is reported, an empty directory passes, and a missing
   directory is refused. So the failing branch runs on Linux and on macOS (BSD `find`) every time.
   This avoids pushing a deliberately failing commit to the pull request: CI runs only on pull
   requests and pushes to `main`.
2. **The XDG directories were not redirected**, unlike in the gate. A process writing through
   them would have written into the real home unseen. They now point inside the empty home,
   without being created.
3. **A home that could not be listed counted as "nothing written"**, in CI and in
   `scripts/gate.sh`. Both now fail. The gate uses `pipefail` for the listing.
4. **The empty directory was created with `mkdir -p`**, which keeps existing contents. It is now
   removed and created afresh.
5. **Post-job steps ran with the empty `HOME`.** The check now points `HOME` back at the real home.
   The workflow comment says steps that write into `HOME` by design must stay outside the two
   steps.
6. **The check was skipped when a test step failed**, so the writes of a failing pull request were
   never listed. It now runs unless the job was cancelled, whenever the empty home was prepared.
7. **The error did not say what to do.** It names the first file written and the likely causes:
   - a test binary without `copperpour_test_home::isolate()` (pointing to `CONTRIBUTING.md`);
   - a doctest that writes per-user files;
   - `COPPERPOUR_TEST_HOME_UNCHANGED=1` left set.
8. **Records that still said CI does not run the check**: `CONTRIBUTING.md`, the crate
   documentation and `docs/BACKLOG.md` are updated. On Windows, nothing checks that the real
   profile is untouched; that is kept in the backlog.

Not changed:
- The schematic viewer's CI job runs only on Windows, where this check does not apply.
- The cache save after a changed `Cargo.lock` has not run with these steps yet. `CARGO_HOME` is set
  explicitly and `HOME` is now restored before post-job steps.

Commit `5793fc5b`. Gate (`logs/gate-ci-home.summary`): 3,815 passed, 0 failed, 115 ignored,
`home-untouched` PASS. CI run 41343986384 passes on all ten jobs. On both Linux and macOS its log
shows:
- "selftest passed: written file reported, empty directory passed, missing directory refused";
- `XDG_CONFIG_HOME` inside the empty home in the test step's environment;
- "nothing written" from the check.

## As shipped, against the plan

Planned (backlog): CI runs the empty-`HOME` check. Shipped on Linux and macOS, with a self-test of
the failing result on every build, the XDG directories redirected, and the same listing-failure fix
in `scripts/gate.sh`. Not shipped: a check on Windows, recorded in `docs/BACKLOG.md`.

Documentation review: this log was checked against the commits, the gate summary and the CI log
by its author, not by an independent reviewer.
