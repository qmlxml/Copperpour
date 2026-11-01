//! Konnect — cdylib entry point for KiCAD plugin integration.
//!
//! When compiled as a cdylib, this crate exposes a C-compatible ABI that
//! a thin Python action plugin (or KiCAD PCM package) can load.

mod config;
mod ffi;
mod transport;

pub use ffi::*;

/// Keep this crate's unit tests out of the developer's home (see
/// `copperpour-test-home`).
#[cfg(test)]
#[ctor::ctor(unsafe)]
fn isolate_test_home() {
    copperpour_test_home::isolate();
}
