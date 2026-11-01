#!/bin/bash
# applydrc.sh PROJDIR(stripped) BASENAME PLAN TAG -- copy project, write plan into board, run refilling DRC, print summary
S="${ROUTER_ASSESS_DIR:?set ROUTER_ASSESS_DIR to a work directory (runs/ and routed/ are created there)}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$S/runs" "$S/routed"
P=$1; B=$2; PLAN=$3; T=$4; O=$S/routed/$T; rm -rf $O; mkdir -p $O; cp $P/$B.kicad_pro $O/
python3 -I $HERE/pcbtool.py apply $P/$B.kicad_pcb $PLAN $O/$B.kicad_pcb
kicad-cli pcb drc --refill-zones --format json --severity-all -o $O/drc.json $O/$B.kicad_pcb >/dev/null 2>&1
echo "$T $(python3 -I $HERE/drcsum.py $O/drc.json)"
