#!/usr/bin/env bash
# CI check that the tests write nothing into the user's home directory.
#
# Why: every test binary points its per-user directories at a shared test
# directory (crates/copperpour-test-home, `isolate()`), so a plain `cargo test`
# never writes into a developer's home. A test binary without that call, or a
# doctest that writes per-user files, would write there again, and nothing else
# reports it. scripts/gate.sh runs the same check locally; this script is the CI
# version (.github/workflows/ci.yml, job `check`, Linux and macOS).
#
# Usage, from GitHub Actions steps (GITHUB_ENV and RUNNER_TEMP are set there):
#   ci_home_untouched.sh selftest   check that `check` fails on a written file
#                                   and passes on an empty directory
#   ci_home_untouched.sh prepare    create an empty home and point HOME and the
#                                   XDG directories at it for the later steps
#   ci_home_untouched.sh check      fail, listing them, if files appeared there;
#                                   then point HOME back at the real home
#
# Not on Windows: there the per-user folders come from the user profile through
# the known-folder API, not from HOME, so this check would see nothing.

set -u

# Exit 0 when directory $1 exists and is empty; 1, listing its contents, when
# something was written there; 2 when it cannot be read.
check_dir() {
    local dir="$1" first
    if [ ! -d "$dir" ]; then
        echo "::error::the empty home $dir does not exist, so the check cannot run"
        return 2
    fi
    if ! first="$(find "$dir" -mindepth 1 -print -quit)"; then
        echo "::error::could not list $dir, so the check cannot run"
        return 2
    fi
    if [ -n "$first" ]; then
        echo "::error::the tests wrote into the empty HOME, first: ${first#"$dir"/}." \
            "A test binary probably lacks copperpour_test_home::isolate() in a" \
            "#[ctor::ctor(unsafe)] function (CONTRIBUTING.md, 'A new test binary must" \
            "keep out of your home directory'), a doctest writes per-user files, or" \
            "COPPERPOUR_TEST_HOME_UNCHANGED=1 is set."
        find "$dir" -mindepth 1 | sed "s|^$dir/||" | sort | head -50
        return 1
    fi
    echo "nothing written under $dir"
    return 0
}

case "${1:-}" in
selftest)
    scratch="$(mktemp -d)" || exit 2
    mkdir "$scratch/empty" "$scratch/written"
    : > "$scratch/written/.written-by-a-test"
    output="$(check_dir "$scratch/written")"
    status=$?
    if [ "$status" -ne 1 ] || ! printf '%s' "$output" | grep -q '.written-by-a-test'; then
        echo "::error::selftest: a written file was not reported (status $status): $output"
        exit 1
    fi
    if ! check_dir "$scratch/empty" > /dev/null; then
        echo "::error::selftest: an empty directory was reported as written"
        exit 1
    fi
    if check_dir "$scratch/missing" > /dev/null; then
        echo "::error::selftest: a missing directory was reported as empty"
        exit 1
    fi
    rm -rf "$scratch"
    echo "selftest passed: written file reported, empty directory passed, missing directory refused"
    ;;
prepare)
    : "${GITHUB_ENV:?prepare runs in GitHub Actions}"
    empty_home="${RUNNER_TEMP:?prepare runs in GitHub Actions}/copperpour-empty-home"
    rm -rf "$empty_home"
    mkdir "$empty_home" || exit 2
    # The XDG directories point inside the empty home without being created,
    # so a process that writes through them is seen.
    {
        echo "CARGO_HOME=${CARGO_HOME:-$HOME/.cargo}"
        echo "RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup}"
        echo "COPPERPOUR_TEST_REAL_HOME=$HOME"
        echo "COPPERPOUR_CI_EMPTY_HOME=$empty_home"
        echo "HOME=$empty_home"
        echo "XDG_CONFIG_HOME=$empty_home/.config"
        echo "XDG_DATA_HOME=$empty_home/.local/share"
        echo "XDG_STATE_HOME=$empty_home/.local/state"
        echo "XDG_CACHE_HOME=$empty_home/.cache"
    } >> "$GITHUB_ENV"
    echo "HOME for the test steps: $empty_home"
    ;;
check)
    empty_home="${COPPERPOUR_CI_EMPTY_HOME:?run prepare first}"
    check_dir "$empty_home"
    status=$?
    # Later steps, and post-job steps such as the cache save, use the real home.
    if [ -n "${GITHUB_ENV:-}" ] && [ -n "${COPPERPOUR_TEST_REAL_HOME:-}" ]; then
        echo "HOME=$COPPERPOUR_TEST_REAL_HOME" >> "$GITHUB_ENV"
    fi
    exit "$status"
    ;;
*)
    echo "usage: $0 selftest|prepare|check" >&2
    exit 2
    ;;
esac
