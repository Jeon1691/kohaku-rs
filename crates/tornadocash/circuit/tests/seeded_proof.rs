//! Pins the proof produced for fixed inputs and a fixed rng seed, so later changes to the artifacts
//! or to how they are loaded cannot silently change what is proven.

mod common;

use kohaku_tornadocash_circuit::{CircuitArtifacts, CircuitInputs, prove};
use rand::{SeedableRng, rngs::ChaCha20Rng};

const SIGNALS_DATA: &str = include_str!("../benches/signals.json");

/// Arbitrary; only needs to stay fixed so the snapshot is reproducible. `ChaCha20Rng`, unlike
/// `StdRng`, promises the same stream across releases.
const SEED: u64 = 139;

// Proving overflows the stack in a debug build.
#[test]
#[ignore = "run with `cargo test --release -- --ignored`"]
fn seeded_proof_is_stable() {
    let inputs: CircuitInputs = serde_json::from_str(SIGNALS_DATA).unwrap();
    let artifacts = common::artifacts();
    let mut rng = ChaCha20Rng::seed_from_u64(SEED);
    let proof = prove(&inputs, &artifacts, &mut rng).unwrap();
    insta::assert_snapshot!(serde_json::to_string_pretty(&proof).unwrap());
}

/// `CircuitArtifacts` is serializable, but the circuit's hash maps make the encoding
/// order-dependent, so re-encoding is checked by what it proves rather than byte for byte.
// Proving overflows the stack in a debug build.
#[test]
#[ignore = "run with `cargo test --release -- --ignored`"]
fn reencoded_artifacts_prove_the_same() {
    let inputs: CircuitInputs = serde_json::from_str(SIGNALS_DATA).unwrap();
    let artifacts = common::artifacts();
    let reencoded =
        CircuitArtifacts::from_bytes(&postcard::to_allocvec(&artifacts).unwrap()).unwrap();
    let proof = prove(&inputs, &artifacts, &mut ChaCha20Rng::seed_from_u64(SEED)).unwrap();
    let again = prove(&inputs, &reencoded, &mut ChaCha20Rng::seed_from_u64(SEED)).unwrap();
    assert_eq!(proof, again);
}
