#!/bin/bash
# runjob.sh EXE BOARD OUTPLAN EXTRA_JSON_FIELDS  -- start a native route job, poll until terminal, print report
S="${ROUTER_ASSESS_DIR:?set ROUTER_ASSESS_DIR to a work directory (runs/ and routed/ are created there)}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$S/runs" "$S/routed"
EXE=$1; B=$2; OUT=$3; EXTRA=${4:-}
d=$(mktemp -d -p $S/runs)
echo "[[\"load_toolset\",{\"name\":\"native_routing\"}],[\"start_native_route_job\",{\"board\":\"$B\",\"output_plan\":\"$OUT\"$EXTRA}]]" > $d/start.json
echo "[[\"load_toolset\",{\"name\":\"native_routing\"}],[\"get_native_route_job\",{\"output_plan\":\"$OUT\"}]]" > $d/poll.json
t0=$(date +%s.%N)
python3 -I $HERE/mcpcall.py $EXE $d/start.json 2>/dev/null | tail -n +3
while true; do
  r=$(python3 -I $HERE/mcpcall.py $EXE $d/poll.json 2>/dev/null | tail -n +4)
  st=$(echo "$r" | python3 -I -c 'import json,sys; print(json.load(sys.stdin).get("state"))' 2>/dev/null)
  case "$st" in RUNNING|STARTING|None|"") sleep 2;; *) break;; esac
done
echo "WALL_S $(python3 -c "import time;print(round(time.time()-$t0,1))")"
echo "$r" > $OUT.status.json
echo "STATE $st"
