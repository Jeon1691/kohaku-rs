# Withdrawal proving

`Withdrawal` proves a canonical Tornado Cash withdrawal; `ProvenWithdrawal` exposes
its call and encoded proof. See the method docs for consuming ownership, errors
and synchronous execution.

Proving needs the circuit artifacts. The bindings do not load them: pass their
postcard-encoded bytes to `CircuitArtifacts.fromBytes`.

With generated bindings loaded and existing `pool`, `note`, `payer`, `recipient`,
`merkleProof` and artifact `bytes`:

```js
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
