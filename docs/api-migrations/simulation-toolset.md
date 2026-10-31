### New `simulation` toolset, error kind and settings (minor release)

New toolset `simulation` with two tools: `check_simulation_readiness` (reports whether a schematic
can be simulated, and why not, without running it) and `run_simulation` (runs one analysis in an
isolated `copperpour sim-worker` process with ngspice and returns compact results in the shared
`outcome` envelope). New error kind `simulation_failed`, with `stage` and `reason`.
`get_installation_info` gains an `ngspice` block (library path, version, code models). New
configuration keys: `ngspice_library`, `ngspice_code_models_dir`, `ngspice_allow_digital_models`
and `ngspice_model_dirs`. A stopped run's `stopped` object may carry `node` (ngspice 44) or
`instance` (ngspice 47) naming where the analysis failed. Nothing existing changes; callers need
no action.
