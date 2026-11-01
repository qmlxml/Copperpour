#[allow(clippy::all, warnings)]
pub mod gen;

pub mod builders;
pub mod client;
mod endpoint;
pub mod footprint_text;
pub mod socket;
pub mod transform;
pub mod types;

pub use client::{
    is_transport_unreachable, unreachable_reason, ApiStatusError, BoardTargetError,
    IpcDocumentObservationError, IpcEditorIdentityUnsupported, IpcFailure, KiCadIpcClient,
    PingOutcome, TransportUnreachable, UnreachableReason,
};
pub use endpoint::redact_endpoint;
pub use socket::{candidate_socket_paths, detect_ipc_address};
pub use types::*;

/// Keep this crate's unit tests out of the developer's home (see
/// `copperpour-test-home`).
#[cfg(test)]
#[ctor::ctor(unsafe)]
fn isolate_test_home() {
    copperpour_test_home::isolate();
}
