"""Assessment helper: strip routed copper from a .kicad_pcb, or insert a native-router plan.

strip IN OUT          removes every top-level (segment ...), (arc ...) and (via ...) block;
                      zones, footprints and graphics are kept.
apply IN PLAN OUT     inserts each plan track as a top-level (segment (start)(end)(width)
                      (layer)(net "name")(uuid)) and each plan via as (via (at)(size)(drill)
                      (layers "F.Cu" "B.Cu")(net "name")(uuid)) before the final ')'.
                      Plan vias are through vias, so layers are the first and last copper layer.
"""
import json, sys, uuid

def blocks(text):
    """Yield (start, end) of top-level child blocks of the root list."""
    i, n, depth, start = 0, len(text), 0, None
    instr = False
    while i < n:
        c = text[i]
        if instr:
            if c == '\\': i += 1
            elif c == '"': instr = False
        elif c == '"': instr = True
        elif c == '(':
            depth += 1
            if depth == 2: start = i
        elif c == ')':
            if depth == 2: yield start, i + 1
            depth -= 1
        i += 1

def strip(src, dst):
    t = open(src).read()
    out, last, removed = [], 0, {}
    for s, e in blocks(t):
        head = t[s+1:s+12].split()[0].rstrip(')')
        if head in ('segment', 'arc', 'via'):
            out.append(t[last:s]); last = e
            removed[head] = removed.get(head, 0) + 1
    out.append(t[last:])
    open(dst, 'w').write(''.join(out))
    print(json.dumps({'removed': removed}))

def apply(src, plan, dst):
    t = open(src).read()
    p = json.load(open(plan))
    cu = []
    for s, e in blocks(t):
        if t[s+1:s+7] == 'layers':
            import re
            cu = [m for m in re.findall(r'\(\d+ "([^"]+\.Cu)"', t[s:e])]
            break
    top, bot = cu[0], cu[-1]
    items = []
    for k in p.get('tracks', []):
        items.append('\t(segment (start %.6f %.6f) (end %.6f %.6f) (width %.6f) (layer "%s") (net "%s") (uuid "%s"))\n'
                     % (k['x1'], k['y1'], k['x2'], k['y2'], k['width'], k['layer'], k['net'], uuid.uuid4()))
    for v in p.get('vias', []):
        items.append('\t(via (at %.6f %.6f) (size %.6f) (drill %.6f) (layers "%s" "%s") (net "%s") (uuid "%s"))\n'
                     % (v['x'], v['y'], v['diameter'], v['drill'], top, bot, v['net'], uuid.uuid4()))
    end = t.rstrip().rfind(')')
    open(dst, 'w').write(t[:end] + ''.join(items) + t[end:])
    print(json.dumps({'tracks': len(p.get('tracks', [])), 'vias': len(p.get('vias', [])), 'layers': [top, bot]}))

if __name__ == '__main__':
    if sys.argv[1] == 'strip': strip(sys.argv[2], sys.argv[3])
    elif sys.argv[1] == 'apply': apply(sys.argv[2], sys.argv[3], sys.argv[4])

def unfill(src, dst):
    """Remove every (filled_polygon ...) inside top-level zones: outlines kept, fill left to KiCad's refill."""
    t = open(src).read(); out = []; last = 0; n = 0
    for s, e in blocks(t):
        if t[s+1:s+5] != 'zone': continue
        z = t[s:e]; i = 0; res = []; 
        while True:
            k = z.find('(filled_polygon', i)
            if k < 0: res.append(z[i:]); break
            d = 0; j = k
            while True:
                if z[j] == '(': d += 1
                elif z[j] == ')':
                    d -= 1
                    if d == 0: break
                j += 1
            res.append(z[i:k]); i = j + 1; n += 1
        out.append(t[last:s]); out.append(''.join(res)); last = e
    out.append(t[last:]); open(dst, 'w').write(''.join(out)); print(json.dumps({'filled_polygons_removed': n}))

if __name__ == '__main__' and sys.argv[1] == 'unfill': unfill(sys.argv[2], sys.argv[3])
