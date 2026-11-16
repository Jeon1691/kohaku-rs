//! The artifacts published in the repository.

mod common;

use std::io::Read;

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

/// Browsers decompress gzip natively, so both files must hold the same artifacts.
#[test]
fn brotli_and_gzip_files_match() {
    let mut from_gzip = Vec::new();
    flate2::read::GzDecoder::new(common::ARTIFACTS_GZ)
        .read_to_end(&mut from_gzip)
        .unwrap();
    assert_eq!(from_gzip, common::decompress_brotli(common::ARTIFACTS_BR));
}
