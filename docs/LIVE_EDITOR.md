# Live schematic editing while KiCad's Schematic Editor is open (Windows)

Contributed by AryanSenthil (fork `AryanSenthil/Konnect`, branch `live-editor`, commit `97dbaa4`,
2026-10-28), merged into this fork in plan step 3 (`docs/development/2026-10-31-00-Fork_Survey.md`).
**Copperpour can edit a schematic while KiCad's Schematic Editor has it open**, on Windows, when the
helper `kicad-editor-bridge.exe` is installed beside `copperpour.exe`.

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
refuses the write as a conflict if the save changed it (`crates/copperpour-sexp/src/writer.rs`,
`prepare_unchanged`). The caller must re-read and plan the edit again; the second attempt is planned
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

and put `kicad-editor-bridge.exe` beside `copperpour.exe`. `tools/kicad-editor-bridge/test-bridge.ps1`
proves the mechanism on a demo copy: an outside edit is lost without the reload and kept with it.
