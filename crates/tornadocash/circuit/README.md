# kohaku-tornadocash-circuit

Tornadocash zk circuits for Kohaku.

Wraps [`websnark-rs`](https://crates.io/crates/websnark-rs) with a simple API for proof generation. The circuit and its proving key are not embedded: the caller loads them as [`CircuitArtifacts`] and passes them to [`prove`].

## Artifacts

The artifacts are in the repository's [`crates/tornadocash/circuit/data`](https://github.com/ethereum/kohaku-rs/tree/master/crates/tornadocash/circuit/data):

| File | Size | Format |
| --- | --- | --- |
| `artifacts.bin.br` | 9.8 MB | brotli |
| `artifacts.bin.gz` | 10.6 MB | gzip, which browsers decompress natively |

Both decompress to the same 26.3 MB, the bytes [`CircuitArtifacts::from_bytes`] takes. Decoding takes a while, so load them once and reuse them.

```rust,no_run
# fn example(inputs: kohaku_tornadocash_circuit::CircuitInputs, decompressed: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
use kohaku_tornadocash_circuit::{CircuitArtifacts, prove};

let artifacts = CircuitArtifacts::from_bytes(&decompressed)?;
let proof = prove(&inputs, &artifacts, &mut rand::rng())?;
# Ok(())
# }
```

Decompressed, the file is the `postcard` encoding of [`CircuitArtifacts`]: the circuit followed by the proving key. The encoding has no version marker, so a change to the struct (for example adding a verifying key) needs new files.

The compressed files were made with brotli 1.2.1 and GNU gzip 1.14 (the one in `nix develop .#ci`). Other versions can produce different, equally valid files.

```sh
brotli -q 11 -w 24 -c artifacts.bin > artifacts.bin.br
gzip -9 -n -c artifacts.bin > artifacts.bin.gz
```

## Benchmarks

Benchmarks were run on a Ryzen 5 3600, 32GB RAM.

| Method | Target           | Time (ms) |
| ------ | ---------------- | --------- |
| prove  | native           | 1,442     |
| prove  | native +parallel | 485.72    |

[`CircuitArtifacts`]: https://docs.rs/kohaku-tornadocash-circuit/latest/kohaku_tornadocash_circuit/struct.CircuitArtifacts.html
[`CircuitArtifacts::from_bytes`]: https://docs.rs/kohaku-tornadocash-circuit/latest/kohaku_tornadocash_circuit/struct.CircuitArtifacts.html#method.from_bytes
[`prove`]: https://docs.rs/kohaku-tornadocash-circuit/latest/kohaku_tornadocash_circuit/fn.prove.html
