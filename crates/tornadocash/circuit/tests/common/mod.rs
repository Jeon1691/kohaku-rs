//! Loads the artifacts from the repository.

// Each test binary uses a different part of this module.
#![allow(dead_code)]

use kohaku_tornadocash_circuit::CircuitArtifacts;

pub const ARTIFACTS_BR: &[u8] = include_bytes!("../../data/artifacts.bin.br");
pub const ARTIFACTS_GZ: &[u8] = include_bytes!("../../data/artifacts.bin.gz");

pub fn decompress_brotli(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    brotli::BrotliDecompress(&mut &data[..], &mut out).unwrap();
    out
}

pub fn artifacts() -> CircuitArtifacts {
    CircuitArtifacts::from_bytes(&decompress_brotli(ARTIFACTS_BR)).unwrap()
}
