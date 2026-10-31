# SPICE simulation: notes from the contributor

These notes come from `docs/SIMULATION_HANDOFF.md` in yuvaltal912-sys's fork (commits up to `e1112a7`,
2026-10-31), merged in plan step 4 (`docs/development/2026-10-31-00-Fork_Survey.md`). The sections about
that fork's own pull requests, next steps and local tools were left out; what remains is the design
goal, where the code is, how to verify it, the traps found while building it, and its known open items.
The full design and its evidence are in `docs/SIMULATION_PLAN.md`.

## The goal

A Copperpour toolset that gives an assistant what KiCad's simulator does: run a schematic through ngspice and return compact, trustworthy results. The first user is low-voltage power distribution boards (inrush, load steps, protection) and analog circuits.

The pipeline: snapshot the saved sheets, export with `kicad-cli`, flatten and vet one deck in Rust, run preflight, load KiCad's libngspice in a hidden single-use `copperpour sim-worker` process, then parse the raw file in Rust.

Decisions already made by the owner:

| ID | Decision |
|---|---|
| D2 | Digital XSPICE models are on. Vetting refuses `d_process` and `d_cosim` |
| D3 | Marked behavioural stand-ins are accepted. Such a result always reads `simulated_circuit: "modified"` |
| Scope | The first version simulates schematics only |
| Platforms | macOS first. Windows and Linux are validated at the end of the series |

## Where things are

| Path | Holds |
|---|---|
| `docs/SIMULATION_PLAN.md` | Design, evidence (E numbers) and open questions (Q numbers) |
| `crates/copperpour-core/src/tools/simulation.rs` | Tool definitions and handlers |
| `crates/copperpour-core/src/simulation/snapshot.rs` | Private copy of the saved sheets |
| `crates/copperpour-core/src/simulation/kicad_netlist.rs`, `net_names.rs` | Pin-to-net map and SPICE names |
| `crates/copperpour-core/src/simulation/flatten.rs` | One vetted deck. This is the security boundary |
| `crates/copperpour-core/src/simulation/element.rs`, `preflight.rs` | Deck reader and checks |
| `crates/copperpour-core/src/simulation/raw.rs` | Raw file parser |
| `crates/copperpour-core/src/simulation/signals.rs`, `measure.rs` | Signal grammar and lookup, and measurements |
| `crates/copperpour-core/src/simulation/worker_client.rs`, `protocol.rs` | Worker supervision and its contract |
| `crates/copperpour/src/sim_worker.rs` | The only FFI. `copperpour-core` stays free of `unsafe` |
| `crates/copperpour/tests/simulation_live.rs` | Ignored tests that need KiCad |
| `crates/copperpour-core/tests/fixtures/simulation/` | Real KiCad and ngspice fixtures, with provenance. `soft_start.sp` and its reference script stand in for E31, and `protection/` and its `reference.py` for E56 |
| `crates/copperpour/assets/skills/kicad-simulation/SKILL.md` | The skill, including the protection stand-ins |

## Verify

Copperpour's pull-request checks (`scripts/gate.sh` runs them), plus the simulation's live tests, which
need ngspice's shared library (`libngspice`) and KiCad's demo projects (`KICAD_DEMOS`):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo test --workspace --locked --lib --tests --no-fail-fast
cargo test --workspace --locked --doc
cargo test -p copperpour --locked --test simulation_live -- --ignored
cargo deny check licenses
```

**Negative controls.** For each guard, break it in the source, run the tests, confirm one fails, and restore it. `GOVERNANCE.md` asks for this. PR 3 to 5 were held to it.

## Traps that already cost time

- `ngSpice_Circ` and `ngSpice_Command` return 0 on failure. `$sim_status`, point counts and coverage are the evidence.
- A halted run writes a complete-looking raw file. Only the worker's own record says it stopped.
- `write` after a failed analysis falls back to the `constants` plot, and the parser refuses that.
- The worker runs in the C locale, so ngspice turns every non-ASCII byte in a node name into `_`. Look signals up by the engine's name, such as `v(/__)`.
- `.probe alli` is a fatal error next to any digital or event node. `run_simulation` leaves it out of those circuits and warns `device_currents_not_saved`.
- A `SIGSEGV` sent from outside does not kill the worker: Rust's handler swallows it. Test crashes with `SIGABRT`.
- ngspice expands `$`, `{` and backticks in commands, and copies `SPICE_SCRIPTS` into a fixed buffer. So `write` takes a relative name and `SPICE_SCRIPTS` is `.`. Never put a path in a command.
- A `bg_halt` sent before the engine's thread starts marks the engine as stopped, and every later one is refused. The worker waits for the thread's own signal before halting.
- ngspice rewrites deck lines in its messages, so matching whole lines cannot keep model text out of errors. `engine_complaints` compares words.
- `ngSpice_nospinit` and `ngSpice_nospiceinit` crash on 44.2 if called before init. The worker shadows those files instead.
- `ngSpice_Circ` stops at the first `.end`, and the first line sent is the title.
- A dangling symlink passes path checks, so test an interpreter by running it.
- Editing Rust with a script breaks after `cargo fmt` reshapes the text. Check the file before rerunning a partly applied edit.
- The default transient step is `stop / 2000`. On a 0.14 s run that is 70 µs: the values stay within 1 percent, but a peak on a flat top lands 0.6 ms early. A smaller step is not always a longer run: 2 µs took 1,019,783 points where 0.5 µs took 281,020.
- `exp()` in a B source or behavioural `R=` is exact only up to an argument of 14, and linear above it, so `exp(ln(1G))` reads 9.3 million. Use `**` or `pow()`.
- A hard-edged control did not run away in any rebuilt short (E98). E57's deck is gone, so do not claim the cause. The failure seen is a lockout chattering, which ends in `no_convergence`.
- Under `uic`, ngspice saves no point at time zero, so `first_point_seconds` is above 0.
- A reviewer's scratch `HOME` moves KiCad's IBIS cache path, so the `ibis` demo only runs from the real home.
- Tests that touch security use nonexistent paths. Engine tests use harmless `echo` and marker files only.

## Known open items

- `.ic` inside a `.subckt` is refused by preflight as `unknown_testbench_node`, or warned as dropped, though ngspice applies it. A false refusal, from PR 4.
- A `.ic` in a dead `.if` still drops device currents under `uic`. It fails loudly.
- `p(REF)` on a subcircuit counts only the pins it shows, so a hidden ground or supply reads short.
- A net named `time` or `frequency`, and `/out#branch`, read `signal_not_found`. No wrong number comes out.
- A part with `Sim.Enable` off reads `as_drawn`. The first PR 7 review reported this from before PR 7, and it has not been rechecked.
- On a partial run, a measurement wholly inside the data reads `window_truncated: true` and counts as failed, as the plan says, though its number is exact.
- Code-model `ic=` under probes, and `bg_halt` when one step takes over a second, have not been tested.
- A value computed from a model parameter can still appear in an engine message, because nothing can recognise it. Worth knowing, not worth more code.
- A real segfault inside libngspice has not been reproduced on 44.2. The crash path is proven with a fake worker and with `SIGABRT` on the real one.
- `.op` has no pause test, so `bg_halt` cannot stop one. It ends at the worker's own deadline.
- `class-d`, `laser_driver` and `v_i_sources` report `engine_messages`. `laser_driver` and `v_i_sources` are true positives. The `class-d` lines were not rechecked after progress notes were filtered out.
- The protection stand-ins are approximations with known edges, all in the skill: the fuse opens in one step with no arc, so inductance ahead of it rings (E97); the reverse stand-in leaks near zero volts because of its 10 mV knee; the eFuse resistance rises near its limit. The independent review of PR 8 found these, not wrong numbers.
- Q17 is an accepted limit: `filesource`, `d_source`, `d_state` and table models read a numeric data file named in the deck.
- Local scratch scripts for mutation runs and dogfooding were never committed. The recipes above replace them.
