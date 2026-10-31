# Live schematic editing while KiCad's Schematic Editor is open (Windows)

Contributed by AryanSenthil (fork `AryanSenthil/Konnect`, branch `live-editor`, commit `97dbaa4`,
2026-10-28), merged into this fork in plan step 3 (`docs/development/2026-10-31-00-Fork_Survey.md`).
**Copperpour can edit a schematic while KiCad's Schematic Editor has it open**, on Windows, when the
helper `kicad-editor-bridge.exe` is installed beside `copperpour.exe` **and `COPPERPOUR_LIVE_EDITOR`
is set to `1`, `on` or `true`**.

**Experimental, off by default** (owner ruling 2026-10-31). The defects listed under "Known defects"
below can lose design data. They are in Windows-only code that has not been compiled or tested since
the merge. Without `COPPERPOUR_LIVE_EDITOR` set to `1`, `on` or `true`, Copperpour refuses schematic writes while the editor
has the file open, as before.

## Known defects (not fixed; recorded in `docs/BACKLOG.md`)

Found by review 2 of plan step 3, from reading the code (not measured):

1. **An agent's edit can be lost without any error.** If the user edits and presses Ctrl+S during the
   1.5 s before the reload, KiCad saves its older content over the agent's write, and the next write
   is allowed because the editor looks clean.
2. **The editor can be left showing stale content with nothing reported.** If the server exits during
   a burst, or the reload fails every time (menu item not found, repeated timeouts), the editor is
   never reloaded; the reload is retried every 1.7 s with only a log warning, later writes still
   succeed, and the user's next save overwrites the agent's edits.
3. **The bridge may click a dialog that is not the Revert confirmation**: it takes the first new
   window of KiCad's process, which can be a dialog the user opened at the same moment, or a tooltip.
4. **Editor windows are matched by file name only** (two projects with `main.kicad_sch` collide; a
   stale lock plus another project's editor of the same name lets the write through) and by English
   window titles (a non-English KiCad is refused with a misleading "stale lock" message).
5. **A conflict never expires**: after the user resolves it and no write follows, an ordinary unsaved
   edit hours later is reported as a conflict whose advice (File > Revert) would discard it.
6. **No timeout on the bridge process**; the reload thread holds the session lock while it reloads,
   so a bridge that hangs blocks all schematic writes.
7. A first write refused because the save changed the file still schedules a reload, which clears
   KiCad's undo history.
8. The bridge's JSON escaping does not cover carriage return or tab, and non-ASCII dialog titles are
   garbled (console code page).
9. Session keys fall back to the raw path spelling when the file cannot be canonicalised.
10. If the session lock is poisoned, the reload thread stops without reporting it.

## Why the write is otherwise refused

KiCad writes `~<file>.kicad_sch.lck` while the Schematic Editor has a schematic open. The editor keeps
its own copy in memory and never notices changes on disk, so its next save silently overwrites
anything written underneath it. KiCad 10's IPC API cannot save or revert a schematic (KiCad 11 adds
`SaveDocument`/`RevertDocument` for schematics), so Copperpour refuses the write and asks you to close
the editor.

## What the feature does

`tools/kicad-editor-bridge/` is a small Windows program (C#, built with the .NET Framework compiler
that ships with Windows) that drives the open editor through its own native File menu: `status`,
`save` (File > Save, wait for the `*` to clear) and `reload` (File > Revert, answer KiCad's
"Confirmation" dialog). It sends menu commands, not keystrokes, so it needs no focus and works while
you're in another window. It only clicks a dialog that appeared because of its own Revert.

`crates/copperpour-sexp/src/writer.rs` gains `set_editor_lock_hook`: when a schematic's KiCad lock is
present, an installed hook may allow the write. With no hook, the write is refused as before.

`crates/copperpour/src/live_editor.rs` installs that hook at startup (Windows, when
`kicad-editor-bridge.exe` sits beside `copperpour.exe`):

1. First write to a locked schematic: save the user's unsaved work, check no dialog is open, allow.
2. Further writes in the burst: allowed while the editor stays clean.
3. `COPPERPOUR_LIVE_EDITOR_DEBOUNCE_MS` (default 1500) after the last write: File > Revert, so the open
   editor shows the result.
4. If the user edits in KiCad between a write and the reload, nothing is reverted and the next write
   is refused, telling them to choose (File > Revert keeps the agent's edit, Ctrl+S keeps theirs).

Refusals carry a reason through every lock message (`schematic_lock_message`): a dialog is open, a
conflict, or no editor window matches the lock (stale lock). `COPPERPOUR_LIVE_EDITOR=0` restores
the refusal; `COPPERPOUR_EDITOR_BRIDGE` overrides the bridge path. The `KONNECT_` names of these
variables are still read.

## Unsaved work in the editor (added when merged)

If the editor has unsaved changes when a write arrives, the bridge saves them first. The edit the
tool is about to write was planned from the file as it was before that save, so writing it would
discard those changes. The writer therefore compares the file before and after the bridge runs, and
refuses the write if the save changed it (`crates/copperpour-sexp/src/writer.rs`,
`prepare_unchanged`, error `ChangedBeforeWrite`, reported to the caller as a conflict in which
nothing was written). The caller must re-read and plan the edit again; the second attempt is planned
from the saved file and goes through. Nothing is lost, at the cost of one refused write.

Operations that need the schematic to be really closed, not just safe to write under, still refuse
while the editor has it open: renaming a project (`rename_project`; KiCad's next save would recreate
the old file name) and journal recovery. Only the content-write paths consult the hook.

## Limits

- Windows only. Matches the editor by window title (`<root sheet> — Schematic Editor`) and English
  menu labels (File / Save / Revert).
- Hierarchical sub-sheet files have no KiCad lock of their own, so writes to them are not
  coordinated (as before this feature).
- PCB editing is unchanged: it already goes through KiCad's live IPC API.
- Annotation (`sch_annotate`) checks for locks itself and does not consult the hook, so it is still
  refused while the editor is open.
- Each write runs the bridge twice (before writing the scratch file and again before replacing the
  file), each as a new process, and a write can wait behind a reload. Expect writes under an open
  editor to take noticeably longer.
- The Rust module `crates/copperpour/src/live_editor.rs` is Windows-only. When it was merged into
  this fork it was not compiled for Windows, because the build machine had no MinGW cross-compiler;
  it was read for API mismatches instead. Build it on Windows before relying on it.

## Build

`cargo build --release -p copperpour` (MSVC or the GNU target with MinGW, protoc and CMake),
then compile the bridge:

```
%WINDIR%\Microsoft.NET\Framework64\v4.0.30319\csc.exe /nologo /optimize /out:kicad-editor-bridge.exe tools\kicad-editor-bridge\KicadEditorBridge.cs
```

and put `kicad-editor-bridge.exe` beside `copperpour.exe`, then set `COPPERPOUR_LIVE_EDITOR=1` (the
feature is off without it). `tools/kicad-editor-bridge/test-bridge.ps1`
proves the mechanism on a demo copy: an outside edit is lost without the reload and kept with it.
