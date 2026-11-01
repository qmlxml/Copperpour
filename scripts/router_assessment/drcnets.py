"""Distinct nets named in a kicad-cli DRC report's unconnected_items (net taken from '[NET]' in item descriptions)."""
import json, re, sys
r = json.load(open(sys.argv[1])); nets = set()
for u in r.get("unconnected_items", []):
    for it in u.get("items", []):
        m = re.search(r"\[(.*?)\]", it.get("description", ""))
        if m: nets.add(m.group(1))
print(len(r.get("unconnected_items", [])), len(nets))
