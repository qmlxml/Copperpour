"""Summarise a native route job status file: per-stage wall time and the completion counters."""
import json, sys
r = json.load(open(sys.argv[1])); rep = r.get("report") or {}
st = {s["stage"]: s for s in rep.get("stages", [])}
rt = st.get("route", {}).get("detail", {}); cl = st.get("clear", {}).get("detail", {})
print(json.dumps({
  "state": r.get("state"), "elapsed_s": round(r.get("elapsed_seconds") or 0, 1),
  "stage_ms": {k: v["wall_ms"] for k, v in st.items()},
  "connections_before": rt.get("connections_before_fanout"), "tasks": rt.get("tasks"),
  "routed": rt.get("routed"), "failed": rt.get("failed"), "connections_after": rt.get("connections_after"),
  "excluded_by_policy": rep.get("excluded_by_policy"), "fanout_sites": rt.get("fanout_sites"),
  "clear_opened": cl.get("opened"), "clear_pieces": cl.get("pieces"), "clear_tracks": cl.get("tracks"), "clear_findings": cl.get("findings"),
  "check": rep.get("check"), "plan_items": r.get("plan_items"), "stopped": rep.get("stopped"), "error": r.get("error")}))
