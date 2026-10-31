"""Resolve git conflict regions in the given files by keeping the HEAD side only.

Why this exists: the fork-merge plan (docs/development/2026-10-31-00-Fork_Survey.md) cherry-picks
forks based on upstream Konnect onto Copperpour. Their commits keep changing tool counts and the
Konnect name in README.md, DEV.md, docs/TROUBLESHOOTING.md, plugin/plugin.json and
packaging/metadata.json. Copperpour's rule is that pull requests leave counts alone (they are
regenerated at release), so those conflict regions resolve to Copperpour's side. `git checkout
--ours <file>` would also discard the contributor's changes that merged without conflict
elsewhere in the same file; this keeps them and drops only the conflicting regions' other side.

It refuses a file whose conflict regions contain anything but HEAD/other blocks (for example a
diff3 base section), and prints each dropped block so the person merging can check that only
counts or names were dropped.

Usage: python3 -I scripts/keep_head_in_conflicts.py FILE... [--dry-run]
"""
import sys


def resolve(text):
    out, dropped, state = [], [], None
    block = []
    for line in text.splitlines(keepends=True):
        if line.startswith("<<<<<<< "):
            if state is not None:
                raise ValueError("nested conflict marker")
            state = "head"
            continue
        if line.startswith("||||||| "):
            raise ValueError("diff3 base section present; resolve by hand")
        if line.startswith("=======") and line.rstrip("\n") == "=======" and state == "head":
            state = "other"
            block = []
            continue
        if line.startswith(">>>>>>> ") and state == "other":
            dropped.append("".join(block))
            state = None
            continue
        if state == "other":
            block.append(line)
        else:
            out.append(line)
    if state is not None:
        raise ValueError("unterminated conflict region")
    return "".join(out), dropped


def main(argv):
    dry = "--dry-run" in argv
    files = [a for a in argv if a != "--dry-run"]
    if not files:
        print(__doc__)
        return 2
    for path in files:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        resolved, dropped = resolve(text)
        print(f"== {path}: {len(dropped)} region(s); other side dropped:")
        for block in dropped:
            for line in block.splitlines():
                print(f"   - {line[:160]}")
        if not dry:
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(resolved)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
