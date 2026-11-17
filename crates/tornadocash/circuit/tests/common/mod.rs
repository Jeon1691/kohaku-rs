//! Loads the artifacts from the repository.

pub const ARTIFACTS_BR: &[u8] = include_bytes!("../../data/artifacts.bin.br");

pub fn decompress_brotli(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    brotli::BrotliDecompress(&mut &data[..], &mut out).unwrap();
    out
}
