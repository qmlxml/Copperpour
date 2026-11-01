# Native-router assessment scripts

Saved from the 2026-11-01 assessment of the J3Techs fork's native autorouter
(`docs/development/2026-11-01-06-J3Techs_router_assessment.md`). They exist so that a later look at
a new revision of that router (or any router that writes a plan of tracks and vias) measures the same
way and gives comparable numbers, instead of re-deriving the method.

What they do:

- `pcbtool.py strip IN OUT` removes every top-level segment, arc and via from a `.kicad_pcb`, keeping
  zones, footprints and graphics; `pcbtool.py apply IN PLAN OUT` writes a routing plan's tracks and
  through vias into a copy of the board, so KiCad's own DRC can judge it.
- `mcpcall.py EXE CALLS.json` is a minimal MCP stdio client: it starts a server binary, initialises
  it and runs a list of tool calls.
- `runjob.sh EXE BOARD OUTPLAN [EXTRA]` starts a `start_native_route_job`, polls it to the end and
  saves its status; `jobsum.py` summarises a status file (stage times, completion counters).
- `applydrc.sh PROJDIR BASENAME PLAN TAG` copies the project, applies a plan and runs
  `kicad-cli pcb drc --refill-zones --severity-all`; `drcsum.py` and `drcnets.py` summarise the DRC
  report (violations by type, unconnected connections and nets).

Set `ROUTER_ASSESS_DIR` to a work directory; `runs/` and `routed/` are created in it. Requires
`kicad-cli` (KiCad 10) and Python 3 (standard library only; run as `python3 -I`). The method,
including why each board was measured with its pour filled and unfilled, is in the assessment
document above.
