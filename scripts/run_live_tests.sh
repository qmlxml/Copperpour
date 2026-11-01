#!/bin/bash
# Run the live tests that need a real KiCad installation: the SPICE simulation tests
# (crates/copperpour/tests/simulation_live.rs) and the DRC parity / SES candidate live tests.
#
# Why this exists: these tests are `#[ignore]`d in the normal gate because they need ngspice's
# shared library, kicad-cli and KiCad's demo projects (`KICAD_DEMOS`). The Arch Linux kicad
# packages install no demos, so they are fetched once, for the installed KiCad version, from
# KiCad's GitLab (only the `demos` folder of the release archive, about 96 MB compressed) into a
# cache directory outside the repository. Typing this by hand each time risks running against the
# wrong demos version, or silently skipping the tests that need them.
#
# Usage: scripts/run_live_tests.sh [simulation|drc|all]   (default: all)
# Environment: KICAD_DEMOS to use existing demos; DEMOS_CACHE (default ~/.cache/copperpour-kicad-demos).
# Writes logs/live-<what>.log; exit status 0 only if every selected test passed.

set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."
what="${1:-all}"
mkdir -p logs
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-line-tables-only}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-line-tables-only}"
export PROTOC="${PROTOC:-$(command -v protoc)}"
export KICAD_CLI="${KICAD_CLI:-$(command -v kicad-cli)}"

if [ -z "${KICAD_DEMOS:-}" ]; then
    version="$("$KICAD_CLI" version | tr -d '[:space:]')"
    cache="${DEMOS_CACHE:-$HOME/.cache/copperpour-kicad-demos}/$version"
    if [ ! -d "$cache/demos" ]; then
        mkdir -p "$cache"
        echo "fetching KiCad $version demos into $cache"
        curl -sSfL -o "$cache/demos.tar.gz" \
            "https://gitlab.com/kicad/code/kicad/-/archive/$version/kicad-$version.tar.gz?path=demos" \
            && tar -xzf "$cache/demos.tar.gz" -C "$cache" --strip-components=1 \
            && rm -f "$cache/demos.tar.gz" || { echo "could not fetch the demos"; exit 2; }
    fi
    export KICAD_DEMOS="$cache/demos"
fi
echo "kicad-cli $("$KICAD_CLI" version), KICAD_DEMOS=$KICAD_DEMOS"

status=0
if [ "$what" = simulation ] || [ "$what" = all ]; then
    cargo test -p copperpour --locked --test simulation_live -- --ignored --test-threads=4 \
        > logs/live-simulation.log 2>&1 || status=1
    grep -E '^test result' logs/live-simulation.log
fi
if [ "$what" = drc ] || [ "$what" = all ]; then
    cargo test -p copperpour-core --locked --lib -- --ignored drc_parse_tests::live candidate_drc_live hierarchy_summary_live \
        > logs/live-drc.log 2>&1 || status=1
    grep -E '^test result' logs/live-drc.log
fi
exit $status
