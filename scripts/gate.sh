#!/bin/bash
# Run Copperpour's local pull-request gate (CONTRIBUTING.md, AGENTS.md) and save a log.
#
# Why this exists: every merge step of the fork-integration plan
# (docs/development/2026-10-31-00-Fork_Survey.md) is checked against the same gate, and the
# result must be comparable with the baseline run on unmodified Copperpour main. Typing the
# commands by hand each time risks omitting one, so a step could "pass" only because a check
# was skipped.
#
# Usage: scripts/gate.sh <label>
#   Writes logs/gate-<label>.log and logs/gate-<label>.summary (one line per check: PASS/FAIL).
#   Exit status is 0 only if every check passed.
#
# The schematic-viewer crate is separate from the workspace and is checked as well.
# --no-fail-fast: without it cargo stops at the first failing test binary, so the rest of the
# workspace is never run and two runs cannot be compared by test count.

set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

label="${1:?usage: scripts/gate.sh <label>}"
mkdir -p logs
log="logs/gate-$label.log"
summary="logs/gate-$label.summary"
: > "$log"
: > "$summary"
export PROTOC="${PROTOC:-$(command -v protoc)}"

# Build directory and disk space.
# On 2026-10-31 a full debug build filled /home to 100% on this machine (target/ was about
# 16 GB by du). The owner then freed space on /home (42 GB free), so the build stays in the
# project's target/ directory. To build elsewhere, set CARGO_TARGET_DIR; a directory on the
# root partition must not be under /var/tmp, where systemd-tmpfiles deletes files unused for
# 30 days.
# Line-table-only debug information roughly halves the build size and does not change test
# results. The gate refuses to start, and stops between checks, when the build partition has
# less than MIN_FREE_GB free: /home also holds the owner's other work.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-line-tables-only}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-line-tables-only}"
MIN_FREE_GB="${MIN_FREE_GB:-10}"
mkdir -p "$CARGO_TARGET_DIR"
if [ ! -d "$CARGO_TARGET_DIR" ] || [ ! -w "$CARGO_TARGET_DIR" ]; then
    echo "gate: build directory $CARGO_TARGET_DIR is missing or not writable" | tee -a "$summary"
    exit 2
fi
free_gb() { df --output=avail -BG "$CARGO_TARGET_DIR" | tail -1 | tr -dc '0-9'; }
check_space() {
    local free
    free=$(free_gb)
    if [ "$free" -lt "$MIN_FREE_GB" ]; then
        echo "ABORT disk: ${free} GB free on the build partition, below ${MIN_FREE_GB} GB" | tee -a "$summary"
        exit 3
    fi
}
check_space

# Home-directory check (2026-11-01). Every test binary points its per-user
# directories at a temporary one (crates/copperpour-test-home). To catch a binary
# that does not, and doctests, which cannot, the checks run with HOME and the XDG
# directories set to an empty directory, and the gate fails if anything is written
# there. RUSTUP_HOME and CARGO_HOME stay on the real ones: without them rustup
# downloads a toolchain into the empty home (measured).
real_home="${HOME:?gate: HOME is not set}"
export RUSTUP_HOME="${RUSTUP_HOME:-$real_home/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$real_home/.cargo}"
gate_home="$(mktemp -d "${TMPDIR:-/tmp}/copperpour-gate-home.XXXXXX")" || {
    echo "gate: could not create a temporary HOME" | tee -a "$summary"
    exit 2
}
# The tests' shared directory (crates/copperpour-test-home) for this run only:
# a fresh one under the build directory, so a run neither depends on what an
# earlier one left nor removes the directory another run of the same checkout
# is using (review 2). Removed when the gate ends, however it ends.
test_root="$(mktemp -d "$CARGO_TARGET_DIR/copperpour-test-home.XXXXXX")" || {
    echo "gate: could not create the tests' shared directory" | tee -a "$summary"
    exit 2
}
export COPPERPOUR_TEST_HOME_ROOT="$test_root"
# The developer's own home, for tests that must find a per-user KiCad
# installation: recorded before HOME changes (review 2).
export COPPERPOUR_TEST_REAL_HOME="$real_home"
cleanup() {
    rm -rf "$test_root"
    # The gate's home is kept for inspection only when the check found
    # something in it.
    [ -d "$gate_home" ] && rmdir "$gate_home" 2>/dev/null
    return 0
}
trap cleanup EXIT
export HOME="$gate_home"
unset XDG_CONFIG_HOME XDG_DATA_HOME XDG_STATE_HOME XDG_CACHE_HOME

status=0
run() {
    local name="$1"; shift
    check_space
    echo "===== $name: $*" >> "$log"
    if "$@" >> "$log" 2>&1; then
        echo "PASS $name" >> "$summary"
    else
        echo "FAIL $name" >> "$summary"
        status=1
    fi
}

run fmt            cargo fmt --all -- --check
run clippy         cargo clippy --workspace --locked --all-targets -- -D warnings
run reliability    cargo test -p copperpour-core --locked --no-fail-fast reliability_contract
run test           cargo test --workspace --locked --no-fail-fast --lib --tests
run doctest        cargo test --workspace --locked --no-fail-fast --doc
run viewer-test    cargo test --locked --manifest-path crates/schematic-viewer/Cargo.toml

# Anything in the gate's home was written by something not isolated.
written="$(cd "$gate_home" && find . -mindepth 1 | sort)"
if [ -z "$written" ]; then
    echo "PASS home-untouched" >> "$summary"
else
    echo "FAIL home-untouched (kept: $gate_home)" >> "$summary"
    echo "===== home-untouched: written under the gate's HOME" >> "$log"
    echo "$written" >> "$log"
    echo "$written" | head -20 | sed 's/^/  /' >> "$summary"
    status=1
fi

# Test-result totals, so two runs can be compared by number as well as by verdict.
grep -E '^test result:' "$log" | awk '{p+=$4; f+=$6; i+=$8} END {printf "TOTAL passed=%d failed=%d ignored=%d\n", p, f, i}' >> "$summary"
grep -E '^test .* FAILED$|^    [a-z_:0-9]+$' "$log" | sed -n '/FAILED/p' | sort -u >> "$summary"
cat "$summary"
exit $status
