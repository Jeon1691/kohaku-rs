# Withdrawal proving

`Withdrawal` proves a canonical Tornado Cash withdrawal; `ProvenWithdrawal` exposes
its call and encoded proof. See the method docs for consuming ownership, errors
and synchronous execution.

Proving needs the circuit artifacts, published in the repository's
[`crates/tornadocash/circuit/data`](https://github.com/ethereum/kohaku-rs/tree/master/crates/tornadocash/circuit/data) as `artifacts.bin.gz` (10.6 MB) and
`artifacts.bin.br` (9.8 MB). Both decompress to 26.3 MB. The bindings do not
download them: fetch one, decompress it and pass the bytes to
`CircuitArtifacts.fromBytes`. Decoding takes a while, so load the artifacts once
and reuse them.

Serve `artifacts.bin.gz` as a plain file, without `Content-Encoding: gzip`.
With that header the browser already decompresses the body, and the
`DecompressionStream` below fails.

With generated bindings loaded and existing `pool`, `note`, `payer`, `recipient`
and `merkleProof`:

```js
const response = await fetch("/artifacts.bin.gz");
if (!response.ok || !response.body) throw new Error(`artifacts: HTTP ${response.status}`);
const decompressed = response.body.pipeThrough(new DecompressionStream("gzip"));
const bytes = new Uint8Array(await new Response(decompressed).arrayBuffer());
// Keep `artifacts` for later proofs, and call `artifacts.free()` when done with it.
const artifacts = CircuitArtifacts.fromBytes(bytes);

const withdrawal = new Withdrawal(pool, note, recipient).withPayer(payer);
const proven = withdrawal.prove(merkleProof, artifacts);
try {
  const call = proven.data();
  const proofBytes = proven.solidityProof();
} finally {
  proven.free();
}
```
