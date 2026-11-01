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
//! isolation. The gate (`scripts/gate.sh`) and CI on Linux and macOS
//! (`scripts/ci_home_untouched.sh`) run the suite with `HOME` set to an empty
//! directory and fail if anything appears in it, which catches a test binary
//! that does not call this, and doctests, which cannot.
//!
//! On Windows `dirs` reads the known-folder API, not `APPDATA` or
//! `LOCALAPPDATA`. The Windows CI runs of 2026-11-01 are consistent with that
//! API resolving the roaming folder under `USERPROFILE` and returning nothing
//! while that folder is missing (`a_relative_home_gives_no_relative_user_directory`
//! failed until `AppData/Roaming` and `AppData/Local` were created under the
//! temporary home); this is inferred from pass and fail, not read from the
//! API's output. So the folders are created there, and `config_dir` and
//! `data_local_dir` are expected to follow this isolation. Not measured on Windows: whether
//! `home_dir` also follows `USERPROFILE`, and whether kicad-cli's cache
//! folder follows `LOCALAPPDATA` now that it exists under the temporary home. `UNCHANGED` is
//! inherited by grandchildren and, if exported in a shell, turns isolation off
//! for the whole run; the gate's empty-home check is what would show it. A
//! process started with its environment cleared and not carrying
//! `COPPERPOUR_TEST_HOME_ROOT` (the simulation worker launched by a test
//! server) has no `HOME` at all, so `dirs` falls back to the account's real
//! home, which the gate's check cannot see; nothing measured writes there.
//!
//! The test `every_test_binary_calls_isolate` (below) runs under plain
//! `cargo test`, so CI catches a test binary added without the call.

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
    let kicad_config = "home/AppData/Roaming/kicad";
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
        // Under the home, where Windows puts them: the known-folder API that
        // `dirs` uses expands `%USERPROFILE%\AppData\…`, and returns nothing
        // when the folder is missing.
        ("APPDATA", "home/AppData/Roaming"),
        ("LOCALAPPDATA", "home/AppData/Local"),
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
    /// Every test binary of the workspace and the schematic viewer calls
    /// `isolate()`: each `tests/*.rs`, and the `src/lib.rs` or `src/main.rs` of
    /// every crate with unit tests. Reads the sources only, so it runs (and
    /// fails, naming the files) under plain `cargo test` and in CI, where the
    /// gate's empty-home check does not run (review 3 of this change).
    #[test]
    fn every_test_binary_calls_isolate() {
        use std::path::Path;
        let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let has_unit_tests = |src: &Path| -> bool {
            let mut stack = vec![src.to_path_buf()];
            while let Some(dir) = stack.pop() {
                for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().is_some_and(|ext| ext == "rs")
                        && std::fs::read_to_string(&path)
                            .is_ok_and(|text| text.contains("#[cfg(test)]"))
                    {
                        return true;
                    }
                }
            }
            false
        };
        let mut missing = Vec::new();
        let mut checked = 0;
        for entry in std::fs::read_dir(crates).unwrap().flatten() {
            let krate = entry.path();
            if krate
                .file_name()
                .is_some_and(|name| name == "copperpour-test-home")
            {
                continue;
            }
            let mut must: Vec<std::path::PathBuf> = std::fs::read_dir(krate.join("tests"))
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
                .collect();
            if has_unit_tests(&krate.join("src")) {
                for root in ["src/lib.rs", "src/main.rs"] {
                    let path = krate.join(root);
                    if path.is_file() {
                        must.push(path);
                    }
                }
            }
            for path in must {
                checked += 1;
                let text = std::fs::read_to_string(&path).unwrap();
                if !text.contains("copperpour_test_home::isolate()") {
                    missing.push(path.strip_prefix(crates).unwrap().display().to_string());
                }
            }
        }
        assert!(
            checked >= 36,
            "only {checked} test binaries found under {crates:?}"
        );
        assert!(
            missing.is_empty(),
            "these test binaries do not call copperpour_test_home::isolate() from a \
             #[ctor::ctor(unsafe)] function (see CONTRIBUTING.md), so their tests write into the \
             developer's home: {missing:#?}"
        );
    }

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
