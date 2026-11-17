//! Decoding the circuit artifacts kept in the repository for tests.

mod common;

use kohaku_tornadocash_circuit::{CircuitArtifacts, CircuitArtifactsError};

// Decoding the 26 MB artifacts takes about 45 s in a debug build.
#[test]
#[ignore = "run with `cargo test --release -- --ignored`"]
fn rejects_trailing_bytes() {
    let mut bytes = common::decompress_brotli(common::ARTIFACTS_BR);
    bytes.push(0);
    assert!(matches!(
        CircuitArtifacts::from_bytes(&bytes),
        Err(CircuitArtifactsError::TrailingBytes(1))
    ));
}

// Decoding the 26 MB artifacts takes about 45 s in a debug build.
#[test]
#[ignore = "run with `cargo test --release -- --ignored`"]
fn rejects_truncated_bytes() {
    let mut bytes = common::decompress_brotli(common::ARTIFACTS_BR);
    bytes.pop();
    assert!(matches!(
        CircuitArtifacts::from_bytes(&bytes),
        Err(CircuitArtifactsError::Decode(_))
    ));
}
