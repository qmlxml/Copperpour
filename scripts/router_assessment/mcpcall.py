"""Minimal MCP stdio client for the assessment: start the server, initialise, run a list of
tool calls given as a JSON file [[name, args], ...], print each result's text."""
import json, subprocess, sys, time
exe, calls = sys.argv[1], json.load(open(sys.argv[2]))
env_cfg = sys.argv[3:]  # extra server args
p = subprocess.Popen([exe] + env_cfg, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
nid = 0
def rpc(method, params=None, notify=False):
    global nid
    msg = {"jsonrpc": "2.0", "method": method}
    if params is not None: msg["params"] = params
    if not notify:
        nid += 1; msg["id"] = nid
    p.stdin.write(json.dumps(msg) + "\n"); p.stdin.flush()
    if notify: return None
    while True:
        line = p.stdout.readline()
        if not line: raise SystemExit("server closed")
        r = json.loads(line)
        if r.get("id") == nid: return r
rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "assess", "version": "0"}})
rpc("notifications/initialized", notify=True)
for name, args in calls:
    t0 = time.time()
    if name == "__list__":
        r = rpc("tools/list", {})
        print(json.dumps([t["name"] for t in r["result"]["tools"]])); continue
    r = rpc("tools/call", {"name": name, "arguments": args})
    dt = time.time() - t0
    res = r.get("result", r)
    text = "\n".join(c.get("text", "") for c in res.get("content", [])) if isinstance(res, dict) else str(res)
    print(f"=== {name} ({dt:.1f}s) isError={res.get('isError') if isinstance(res, dict) else None}")
    print(text[:20000])
p.stdin.close(); p.wait(timeout=30)
