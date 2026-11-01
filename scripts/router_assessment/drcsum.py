"""Summarise a kicad-cli DRC JSON report: violation counts by type and severity, unconnected count."""
import json, sys, collections
r = json.load(open(sys.argv[1]))
c = collections.Counter((v['type'], v['severity']) for v in r.get('violations', []))
u = len(r.get('unconnected_items', []))
print(json.dumps({'unconnected': u, 'violations': {f'{t}/{s}': n for (t, s), n in sorted(c.items())}}))
