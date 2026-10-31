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

status=0
run() {
    local name="$1"; shift
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

# Test-result totals, so two runs can be compared by number as well as by verdict.
grep -E '^test result:' "$log" | awk '{p+=$4; f+=$6; i+=$8} END {printf "TOTAL passed=%d failed=%d ignored=%d\n", p, f, i}' >> "$summary"
grep -E '^test .* FAILED$|^    [a-z_:0-9]+$' "$log" | sed -n '/FAILED/p' | sort -u >> "$summary"
cat "$summary"
exit $status
