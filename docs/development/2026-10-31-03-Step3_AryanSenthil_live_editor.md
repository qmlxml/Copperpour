# Step 3 — `AryanSenthil/Konnect`: edit schematics while KiCad's Schematic Editor is open

Plan: `2026-10-31-00-Fork_Survey.md`, section 7. Feature description: `docs/LIVE_EDITOR.md`.

## Merge

Cherry-picked `97dbaa4` (branch `live-editor`) as `a21e1ed`, author kept. The fork was based on
upstream Konnect before Copperpour renamed its crates, so the merge adapted it; these changes are
part of `a21e1ed`:
- `konnect_sexp::` → `copperpour_sexp::` in the five changed files; a message "Konnect did not modify
  it" → "Copperpour did not modify it" (handler.rs), matching Copperpour's other lock messages.
- Environment variables read through `copperpour_core::app_dirs::env_var`, so
  `COPPERPOUR_LIVE_EDITOR`, `COPPERPOUR_LIVE_EDITOR_DEBOUNCE_MS` and `COPPERPOUR_EDITOR_BRIDGE` are
  read first and the `KONNECT_` names still work, as for Copperpour's other variables.
- `konnect.exe` → `copperpour.exe` in messages and documentation; thread name; comments that called
  the code "the fork" or "upstream".
- `FORK.md`, which described AryanSenthil's fork, moved to `docs/LIVE_EDITOR.md` and rewritten as
  this feature's documentation with attribution.

The commit's README changes were not in this commit (the README conflicts came from the six
upstream README commits, which cherry-picking leaves out).

Not compiled for Windows: `crates/copperpour/src/live_editor.rs` is `#[cfg(windows)]`. A
`cargo check --target x86_64-pc-windows-gnu` was attempted and stopped in the build script of
`nng-sys`, which needs the MinGW C compiler (`x86_64-w64-mingw32-gcc`); installing it needs root.
Review 1 read the module for API mismatches and found none.

Gate (`logs/gate-step3-merge.summary`): tests as the baseline plus earlier steps (3,163 passed, the
known failure), but clippy failed on two needless borrows in the contributed code (rust 1.96
`needless_borrows_for_generic_args`). Fixed in `9c822b7`; clippy then passes.

## Review 1 (implementation) and fix pass 1

With no hook installed (Linux, macOS, Windows without the bridge), behaviour is unchanged; the
bridge's commands and JSON fields match what the Rust side reads. Findings and what was done:

1. **The user's unsaved work was lost** (defect, inherited from `97dbaa4`). The writers
   (`write_atomic_if_unchanged`, `transact_atomic`) read and compare the file before the hook runs;
   the hook's File > Save then wrote the user's unsaved changes to disk, and the write replaced them
   with content planned from the older file. Read-back passed and the later reload showed the result
   without the user's changes. Fixed: `write_atomic_unlocked_with` reads the file before the first
   lock check and refuses with a conflict if the check (which may run the hook) changed it; the same
   comparison is repeated at the second check before the rename (`prepare_unchanged`). The function
   that makes the editor safe is now a parameter, so tests can stand in for the hook (which is set
   once per process and cannot be installed in tests). Tests:
   `a_write_is_refused_when_making_the_editor_safe_changed_the_file`,
   `a_write_proceeds_when_making_the_editor_safe_changed_nothing`,
   `a_change_between_the_two_checks_also_refuses_the_write`. Control: with the comparison disabled,
   the first and third fail.
2. **`rename_project` and journal recovery ran while KiCad had the file open** (defect). The
   contribution put the hook into `ensure_kicad_design_document_is_closed`, which those callers use to
   require the document closed. The hook is now consulted only by the content-write paths, through
   the new private `ensure_closed_or_prepared_for_write`; `ensure_kicad_design_document_is_closed`
   is strict again. Test `the_strict_check_refuses_an_observed_schematic_lock`.
3. **Annotation does not consult the hook.** Documented as a limit in `docs/LIVE_EDITOR.md`.
4. **Two matching editor windows were reported as a stale lock.** `prepare` now reports the
   bridge's own reason when `status` is not `ok` (e.g. `ambiguous_editor`). Not compiled here.
5. **Two bridge processes per write; the session mutex is held across bridge runs.** Documented as a
   limit.
6. **An old refusal reason could be shown** after a later attempt failed on an unreadable lock. The
   recorded reason is now forgotten when the hook is not consulted (`forget_editor_lock_refusal`).
7. **"Konnect" in two user-facing messages** in `live_editor.rs`: now "Copperpour".

Not covered by tests: anything that needs the hook installed or the Windows bridge (findings 4
and 6, and the bridge itself). Tests cannot install the process-wide hook without affecting every
other test in the process.

Gate after fix pass 1 (`logs/gate-step3-fix1.summary`): 3,167 passed (+4), 1 failed (the known
baseline failure), 60 ignored; clippy passes.
Committed as `48d4d93`.

## Review 2 (robustness) and fix pass 2

Review 2 found ten problems, nearly all in the Windows-only code (the C# bridge and
`live_editor.rs`), which cannot be compiled, run or tested on this machine. Two can lose design data
without any error (the user's Ctrl+S before the reload overwrites the agent's write; a server exit or
a reload that always fails leaves the editor stale with nothing reported). They are listed in
`docs/LIVE_EDITOR.md`, "Known defects".

**Owner ruling 2026-10-31** (both models recommended it): keep the merged code, make the feature
opt-in, document the known defects next to the setting, finish the review cycle on the shared Rust
parts, and record the Windows defects as deferred work. Options not chosen: fixing all ten without
being able to compile or test them; removing step 3; pausing step 3 with the feature still on by
default.

Done:
- `live_editor.rs` installs the hook only when `COPPERPOUR_LIVE_EDITOR` is `1`, `on` or `true` (it was
  on whenever the bridge was found); the log says it is experimental. Not compiled here.
- `docs/LIVE_EDITOR.md` says the feature is experimental and off by default and lists the ten known
  defects. New `docs/BACKLOG.md` records them with a closing condition, together with the other work
  deferred so far in this merge.
- Finding 7 (shared Rust): the guard compared against a fresh read taken inside the write, not the
  text the edit was planned from. `write_atomic_if_unchanged` and `transact_atomic` now pass the text
  they planned from (`write_atomic_unlocked_planned`). Test
  `a_conditional_write_compares_against_the_planned_from_text`.

Gate after fix pass 2 (`logs/gate-step3-fix2.summary`): 3,168 passed (+1), 1 failed (the known
baseline failure), 60 ignored.
Committed as `f81fe5a`.

## Review 3 (intent and liveness) and fix pass 3

Review 3 confirmed that users who leave the feature off get the same lock behaviour, error kinds and
messages as before the step; line endings, BOMs and normalisation cannot cause the new guard to
refuse (the conditional writers already required byte-equal text); and that no new refusal is
permanent (a retry re-reads the file). Findings and what was done:

1. **A write refused before anything was written was reported as "may have been written"**
   (likely defect, reaches every user). `replace_if_unchanged` (crates/copperpour-core/src/tools/mod.rs)
   relies on `transact_atomic` returning `Conflict` only when the read-back after a write fails, and
   reports that as `MutationOutcomeUncertain`. The new guard also returned `Conflict`. Fixed: a new
   error `SexpError::ChangedBeforeWrite` (crates/copperpour-sexp/src/error.rs), always raised before
   anything is written; `replace_outcome` reports it exactly like a file that changed since the read
   ("wrote nothing. Reload it and retry"). `Conflict` keeps its meaning for the 29 other places that
   match it. Its own message says nothing was written, so callers that do not match it get a clear
   handler error. Test `a_change_before_the_write_is_reported_as_nothing_written`.
2. **Multi-file transactions compared against a fresh read; recovery could consult the hook.**
   `apply_entry` (crates/copperpour-sexp/src/transaction.rs) now writes against each entry's before
   image, and recovery uses new strict writers (`write_atomic_unlocked_strict`,
   `write_new_atomic_unlocked_strict`) that never consult the hook. Test
   `an_entry_whose_file_changed_after_verification_is_not_overwritten`.
3. **A stale hook refusal reason could be shown for a strict refusal.** The recorded reason is now
   forgotten whenever the write path finds no lock, and the server's lock message
   (crates/copperpour-core/src/mcp/handler.rs) is chosen from the error itself
   (`refused_by_live_editor`: `inspection_error` starting "live editor: "), not from whether a hook
   is installed. Test `only_a_refusal_by_the_live_editor_carries_its_reason`.
4. **The documentation said only `=1` enables the feature;** the code also accepts `on` and `true`.
   `docs/LIVE_EDITOR.md` now says so, and its build section says the feature is off without the
   setting.
5. **Backlog:** the closing condition "compiled in CI" could not be met, because Copperpour has no
   Windows CI job. Changed to "tested on a named machine, result recorded", and a separate entry
   added for the missing CI job.

Gate after fix pass 3 (`logs/gate-step3-fix3.summary`): 3,171 passed (+3), 1 failed (the known
baseline failure), 60 ignored.
