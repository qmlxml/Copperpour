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
//! Remaining gaps, stated rather than hidden: on Windows `dirs` reads the
//! known-folder API and ignores `USERPROFILE` and `APPDATA` for `home_dir`,
//! `config_dir` and `data_local_dir`, so Copperpour code that calls those
//! through `app_dirs` is redirected only where it also honours a variable set
//! here (`COPPERPOUR_STATE_DIR`, `APPDATA` read directly); and kicad-cli on
//! Windows may still use the real profile's cache folder. `UNCHANGED` is
//! inherited by grandchildren and, if exported in a shell, turns isolation off
//! for the whole run; the gate's empty-home check is what would show it.

use std::path::{Path, PathBuf};

/// Set to `1` in a child process whose home-related variables the test sets
/// itself; [`isolate`] then leaves them alone.
pub const UNCHANGED: &str = "COPPERPOUR_TEST_HOME_UNCHANGED";

/// The developer's own home, as it was before [`isolate`] changed `HOME`.
/// For tests that must find a per-user KiCad installation or KiCad's real
/// model cache (review 1 of this change): only those read it.
pub const REAL_HOME: &str = "COPPERPOUR_TEST_REAL_HOME";

/// The shared directory's path, exported by [`isolate`] so that child
/// processes (including a copy of the test binary run from elsewhere) use the
/// same one, and set by `scripts/gate.sh` to a directory of its own per run
/// (review 2 of this change).
pub const ROOT: &str = "COPPERPOUR_TEST_HOME_ROOT";

/// The directory test processes share: [`ROOT`] when it is set to an absolute
/// path; else `<target>/copperpour-test-home`, where `<target>` is the nearest
/// directory above the test binary that holds Cargo's `CACHEDIR.TAG` (so
/// `cargo test --target <triple>` and a custom `CARGO_TARGET_DIR` find the same
/// one), else the parent of the profile directory above the binary's `deps`.
/// Under the build directory rather than the system temporary
/// directory, so it belongs to this user and checkout: a fixed name in a shared
/// `/tmp` could be created first by another user, even as a link into this
/// user's home (review 1). When neither applies, a directory named after the
/// user and this process is used, created fresh.
pub fn root() -> PathBuf {
    if let Some(root) = std::env::var_os(ROOT)
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
    {
        return root;
    }
    // The build directory: the nearest ancestor holding Cargo's
    // `CACHEDIR.TAG`, else the parent of the profile directory above `deps`
    // (`<target>[/<triple>]/<profile>/deps/<binary>`); Cargo does not always
    // write the tag (measured: not in this checkout's target directory).
    let target = std::env::current_exe().ok().and_then(|exe| {
        exe.ancestors()
            .skip(1)
            .find(|dir| dir.join("CACHEDIR.TAG").is_file())
            .map(Path::to_path_buf)
            .or_else(|| {
                let deps = exe.parent()?;
                (deps.file_name()? == "deps")
                    .then(|| deps.parent()?.parent().map(Path::to_path_buf))?
            })
    });
    target
        .map(|target| target.join("copperpour-test-home"))
        .unwrap_or_else(|| {
            let user = std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "user".into());
            std::env::temp_dir().join(format!(
                "copperpour-test-home-{user}-{}",
                std::process::id()
            ))
        })
}

/// Point `HOME`, the XDG directories, the Windows profile folders, KiCad's
/// configuration and documents directories and `COPPERPOUR_STATE_DIR` at
/// [`root`], keeping `RUSTUP_HOME` and `CARGO_HOME` where they were (without
/// that, a child `cargo` or `rustup` looks for its toolchain under the new home
/// and downloads one there; measured), and recording the real home in
/// [`REAL_HOME`]. Call it only from a constructor that runs before any thread.
///
/// Does nothing when [`UNCHANGED`] is set to `1`: a test that re-runs its own
/// binary with a deliberate `HOME` (relative, unset, or its own temporary
/// one) sets it, or this would overwrite the very value under test.
pub fn isolate() {
    if std::env::var_os(UNCHANGED).is_some_and(|value| value == "1") {
        return;
    }
    let root = root();
    std::env::set_var(ROOT, &root);
    // Only an absolute home is trusted: a relative one would pin the toolchain
    // relative to the working directory (review 1).
    if let Some(real_home) = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
    {
        keep(&real_home, "RUSTUP_HOME", ".rustup");
        keep(&real_home, "CARGO_HOME", ".cargo");
        if std::env::var_os(REAL_HOME).is_none() {
            std::env::set_var(REAL_HOME, &real_home);
        }
    }
    // KiCad appends its version folder (`10.0`) to KICAD_CONFIG_HOME
    // (measured with kicad-cli 10.0.7), so it is set to the same `kicad` base
    // Copperpour reads KiCad's global tables from (`kicad_config_base`), and
    // the two read one configuration, as in a real home.
    #[cfg(target_os = "windows")]
    let kicad_config = "appdata/roaming/kicad";
    #[cfg(target_os = "macos")]
    let kicad_config = "home/Library/Preferences/kicad";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let kicad_config = "home/.config/kicad";
    let dirs = [
        ("HOME", "home"),
        ("USERPROFILE", "home"),
        ("XDG_CONFIG_HOME", "home/.config"),
        ("XDG_DATA_HOME", "home/.local/share"),
        ("XDG_STATE_HOME", "home/.local/state"),
        ("XDG_CACHE_HOME", "home/.cache"),
        ("APPDATA", "appdata/roaming"),
        ("LOCALAPPDATA", "appdata/local"),
        ("KICAD_CONFIG_HOME", kicad_config),
        // On Windows kicad-cli would otherwise use the real Documents folder.
        ("KICAD_DOCUMENTS_HOME", "home/Documents/KiCad"),
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
            // A real home already recorded (the gate sets it) is kept.
            let expected_real = std::env::var_os(super::REAL_HOME)
                .map(std::path::PathBuf::from)
                .or_else(|| real_home.clone().filter(|home| home.is_absolute()));
            super::isolate();
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            assert!(home.starts_with(super::root()), "{home:?}");
            assert_eq!(
                std::env::var_os(super::ROOT).map(std::path::PathBuf::from),
                Some(super::root())
            );
            assert_eq!(
                std::env::var_os(super::REAL_HOME).map(std::path::PathBuf::from),
                expected_real
            );
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
