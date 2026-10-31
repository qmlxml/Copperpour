#!/bin/bash
# Measure what the installed kicad-cli reports for the schematic-parity DRC check, with and
# without a root schematic beside the board.
#
# Why this exists: Copperpour's `apply_parity_evidence` (crates/copperpour-core/src/tools/cli.rs)
# decides whether an empty `schematic_parity` array is a checked zero or "not checked". It relies
# on kicad-cli printing "Failed to fetch schematic netlist for parity tests" (searched on stderr
# and stdout) when it had no schematic to compare, and on a file check when it prints nothing. A contributed change (halthinks, 2026-10-20) says KiCad 10.0.7 can stay
# silent instead. Which is true decides whether the code's fallback (checking for the file
# itself) is needed, and whether the live test that expects the message can pass. The answer can
# change with each KiCad release, so it is measured, not assumed.
#
# Usage: scripts/measure_parity_stderr.sh [board.kicad_pcb]
#   Default board: crates/copperpour-core/tests/fixtures/board_outline_kicad10.kicad_pcb (no
#   schematic of its own, so the first two cases equal the third). test.kicad_pcb, which has a
#   schematic, is rejected by kicad-cli 10.0.7 (exit 3).
#   Runs these cases in temporary directories: board with its schematic and project, board with
#   its schematic only, the board alone, and the board beside a root schematic that exists but
#   cannot be used (empty file, unparsable text, unreadable permissions, a directory). Prints, per case, the exit status, whether the
#   message appeared on stderr, and the number of entries in "schematic_parity".
# Requires: kicad-cli on PATH (or KICAD_CLI), python3.

set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."
cli="${KICAD_CLI:-kicad-cli}"
board="${1:-crates/copperpour-core/tests/fixtures/board_outline_kicad10.kicad_pcb}"
stem="$(basename "${board%.kicad_pcb}")"
src_dir="$(dirname "$board")"
msg="Failed to fetch schematic netlist for parity tests"

echo "kicad-cli $("$cli" version)  board: $board"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

run_case() {
    local name="$1"; shift
    local dir="$work/$name"
    mkdir -p "$dir"
    cp "$board" "$dir/"
    for extra in "$@"; do
        [ -f "$src_dir/$stem.$extra" ] && cp "$src_dir/$stem.$extra" "$dir/"
    done
    local report="$dir/report.json"
    [ -n "${SETUP:-}" ] && (cd "$dir" && eval "$SETUP")
    "$cli" pcb drc --schematic-parity --format json --output "$report" "$dir/$stem.kicad_pcb" \
        > "$dir/stdout" 2> "$dir/stderr"
    local rc=$?
    local said="no"
    grep -qF "$msg" "$dir/stderr" && said="stderr"
    grep -qF "$msg" "$dir/stdout" && said="${said/no/}stdout"
    local count
    count=$(python3 -I -c 'import json,sys
d=json.load(open(sys.argv[1])); p=d.get("schematic_parity")
print("absent" if p is None else len(p))' "$report" 2>/dev/null || echo "no-report")
    printf "%-22s exit=%s  message=%s  schematic_parity=%s\n" "$name" "$rc" "$said" "$count"
}

run_case with-sch-and-pro kicad_sch kicad_pro
run_case with-sch-only kicad_sch
run_case lone-board
SETUP=": > $stem.kicad_sch" run_case empty-root
SETUP="echo 'not a schematic' > $stem.kicad_sch" run_case garbage-root
SETUP="echo '(kicad_sch (version 20250114) (generator x)' > $stem.kicad_sch" run_case truncated-root
SETUP="echo '(kicad_sch)' > $stem.kicad_sch; chmod 000 $stem.kicad_sch" run_case unreadable-root
SETUP="mkdir $stem.kicad_sch" run_case directory-root
