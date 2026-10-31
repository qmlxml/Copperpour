#!/bin/bash
# Measure whether kicad-cli applies a project's DRC rules to a board copied under another name.
#
# Why this exists: `apply_specctra_ses` (crates/copperpour-core/src/tools/pcb_routing.rs) writes
# the routed result to a new candidate .kicad_pcb and runs DRC on it. KiCad reads rule severities,
# clearances and net classes from <board stem>.kicad_pro (and custom rules from
# <board stem>.kicad_dru). A candidate with a different stem and no project file of its own is
# therefore checked against KiCad's defaults. Measured 2026-10-31 on kicad-cli 10.0.7: 0
# violations for the source, 1 for the candidate alone, 0 after copying the project file. Any
# change to how the candidate is checked must be re-measured with this script.
#
# Usage: scripts/measure_candidate_drc_rules.sh [board.kicad_pcb]
#   Default: crates/copperpour-core/tests/fixtures/netclass_diff_pair_kicad10.kicad_pcb (with its
#   .kicad_pro). Sets the project's invalid_outline severity to "ignore", then counts violations
#   for: the source beside its project; the board copied as candidate.kicad_pcb alone; the
#   candidate with the project copied as candidate.kicad_pro.
# Requires: kicad-cli on PATH (or KICAD_CLI), python3.

set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."
cli="${KICAD_CLI:-kicad-cli}"
board="${1:-crates/copperpour-core/tests/fixtures/netclass_diff_pair_kicad10.kicad_pcb}"
stem="$(basename "${board%.kicad_pcb}")"
project="${board%.kicad_pcb}.kicad_pro"
[ -f "$project" ] || { echo "no project file $project"; exit 2; }

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp "$board" "$project" "$work/"
python3 -I - "$work/$stem.kicad_pro" <<'EOF'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
d.setdefault("board", {}).setdefault("design_settings", {}).setdefault("rule_severities", {})["invalid_outline"] = "ignore"
json.dump(d, open(p, "w"), indent=2)
EOF

count() {
    "$cli" pcb drc --format json --output "$work/r.json" "$1" > /dev/null 2>&1
    python3 -I -c 'import json,sys
d=json.load(open(sys.argv[1])); v=d["violations"]
print(len(v), sorted({x["type"] for x in v}))' "$work/r.json" 2>/dev/null || echo "no report"
}

echo "kicad-cli $("$cli" version)  board: $board"
printf "%-34s %s\n" "source beside its project:" "$(count "$work/$stem.kicad_pcb")"
cp "$work/$stem.kicad_pcb" "$work/candidate.kicad_pcb"
printf "%-34s %s\n" "candidate alone:" "$(count "$work/candidate.kicad_pcb")"
cp "$work/$stem.kicad_pro" "$work/candidate.kicad_pro"
printf "%-34s %s\n" "candidate with copied project:" "$(count "$work/candidate.kicad_pcb")"
