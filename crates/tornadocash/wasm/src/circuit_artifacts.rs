use kohaku_tornadocash::CircuitArtifacts as CoreCircuitArtifacts;
use wasm_bindgen::prelude::*;

/// The Tornado Cash withdrawal circuit and its proving key.
///
/// Decoding takes a while, so load this once and reuse it for every proof.
/// See [`kohaku_tornadocash::CircuitArtifacts`] for more details.
#[wasm_bindgen]
pub struct CircuitArtifacts {
    pub(crate) inner: CoreCircuitArtifacts,
}

#[wasm_bindgen]
impl CircuitArtifacts {
    /// Decode the artifacts from their decompressed bytes, e.g. `artifacts.bin.gz` after a
    /// `DecompressionStream("gzip")`.
    ///
    /// # Errors
    /// Throws if the bytes are not encoded circuit artifacts.
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, JsError> {
        Ok(Self {
            inner: CoreCircuitArtifacts::from_bytes(bytes)?,
        })
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
pub(crate) mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    /// Read from the repository: the wasm32 tests only build from a repository checkout.
    const ARTIFACTS_BR: &[u8] = include_bytes!("../../circuit/data/artifacts.bin.br");

    pub(crate) fn artifacts() -> CircuitArtifacts {
        let mut bytes = Vec::new();
        brotli::BrotliDecompress(&mut &ARTIFACTS_BR[..], &mut bytes).unwrap();
        CircuitArtifacts::from_bytes(&bytes).unwrap()
    }

    #[wasm_bindgen_test]
    fn rejects_bytes_that_are_not_artifacts() {
        assert!(CircuitArtifacts::from_bytes(&[0xff; 8]).is_err());
    }
}
