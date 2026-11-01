//! Keeps tests out of the developer's own per-user directories.
//!
//! Measured 2026-11-01: a plain `cargo test` of the workspace wrote into the
//! real home about 2,300 document lock files per run
//! (`~/.local/share/konnect/locks`, never deleted by design), the call log and
//! EasyEDA cache under `~/.copperpour`, run records of spawned servers,
//! Freerouting's state directory, and, through kicad-cli started by tests,
//! KiCad's own user configuration (`~/.config/kicad/10.0`).
//!
//! Every test binary calls [`isolate`] from a `ctor` constructor, before any
//! test thread starts, so setting environment variables there is not a race.
//! The child processes tests start (kicad-cli, the server binary) inherit the
//! isolation. The gate (`scripts/gate.sh`) runs the suite with `HOME` set to an
//! empty directory and fails if anything appears in it, which catches a test
//! binary that does not call this, and doctests, which cannot.
//!
//! Remaining gap, stated rather than hidden: on Windows `dirs` reads the
//! known-folder API, not these variables, for `data_local_dir`, which only
//! `native_specctra_bridge.rs` uses for state.

use std::path::{Path, PathBuf};

/// Set to `1` in a child process whose home-related variables the test sets
/// itself; [`isolate`] then leaves them alone.
pub const UNCHANGED: &str = "COPPERPOUR_TEST_HOME_UNCHANGED";

/// The directory every test process shares: `<temp>/copperpour-test-home`.
/// Shared rather than per process, so repeated runs reuse it instead of
/// leaving one directory per process; document locks in it keep working
/// across processes as they do in a real home.
pub fn root() -> PathBuf {
    std::env::temp_dir().join("copperpour-test-home")
}

/// Point `HOME`, the XDG directories, the Windows profile folders, KiCad's
/// configuration directory and `COPPERPOUR_STATE_DIR` at [`root`], keeping
/// `RUSTUP_HOME` and `CARGO_HOME` where they were (without that, a child
/// `cargo` or `rustup` looks for its toolchain under the new home and
/// downloads one there; measured). Call it only from a constructor that runs
/// before any thread.
///
/// Does nothing when [`UNCHANGED`] is set to `1`: a test that re-runs its own
/// binary with a deliberate `HOME` (relative, unset, or its own temporary
/// one) sets it, or this would overwrite the very value under test.
pub fn isolate() {
    if std::env::var_os(UNCHANGED).is_some_and(|value| value == "1") {
        return;
    }
    let root = root();
    if let Some(real_home) = std::env::var_os("HOME").map(PathBuf::from) {
        keep(&real_home, "RUSTUP_HOME", ".rustup");
        keep(&real_home, "CARGO_HOME", ".cargo");
    }
    let dirs = [
        ("HOME", "home"),
        ("USERPROFILE", "home"),
        ("XDG_CONFIG_HOME", "home/.config"),
        ("XDG_DATA_HOME", "home/.local/share"),
        ("XDG_STATE_HOME", "home/.local/state"),
        ("XDG_CACHE_HOME", "home/.cache"),
        ("APPDATA", "appdata/roaming"),
        ("LOCALAPPDATA", "appdata/local"),
        ("KICAD_CONFIG_HOME", "kicad-config"),
        ("COPPERPOUR_STATE_DIR", "copperpour-state"),
    ];
    for (variable, relative) in dirs {
        let path = root.join(relative);
        // A directory that cannot be created leaves the variable pointing at
        // it anyway: a test then fails loudly instead of writing to the real
        // home.
        let _ = std::fs::create_dir_all(&path);
        std::env::set_var(variable, &path);
    }
}

/// Set `variable` to `real_home/leaf` when it is unset and that directory
/// exists.
fn keep(real_home: &Path, variable: &str, leaf: &str) {
    let path = real_home.join(leaf);
    if std::env::var_os(variable).is_none() && path.is_dir() {
        std::env::set_var(variable, path);
    }
}

#[cfg(test)]
mod tests {
    /// `isolate` changes the process environment, so it runs in a child: this
    /// test binary re-runs itself with a marker variable, and the child checks
    /// what `isolate` set.
    #[test]
    fn isolate_points_home_away_and_keeps_the_toolchain() {
        if std::env::var_os("COPPERPOUR_TEST_HOME_CHILD").is_some() {
            let real_home = std::env::var_os("HOME").map(std::path::PathBuf::from);
            super::isolate();
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            assert!(home.starts_with(super::root()), "{home:?}");
            assert_ne!(Some(home.clone()), real_home);
            for variable in ["XDG_DATA_HOME", "COPPERPOUR_STATE_DIR", "KICAD_CONFIG_HOME"] {
                let path = std::path::PathBuf::from(std::env::var_os(variable).unwrap());
                assert!(path.starts_with(super::root()), "{variable}: {path:?}");
                assert!(path.is_dir(), "{variable}: {path:?}");
            }
            if let Some(real) = real_home.filter(|home| home.join(".rustup").is_dir()) {
                assert_eq!(
                    std::env::var_os("RUSTUP_HOME").map(std::path::PathBuf::from),
                    Some(real.join(".rustup"))
                );
            }
            return;
        }
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::isolate_points_home_away_and_keeps_the_toolchain",
            ])
            .env("COPPERPOUR_TEST_HOME_CHILD", "1")
            .env_remove("RUSTUP_HOME")
            .status()
            .unwrap();
        assert!(status.success());
    }
}
