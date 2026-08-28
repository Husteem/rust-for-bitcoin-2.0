# Lab 04 — Native P2WPKH

## Commands used

I ran the test suite for native SegWit P2WPKH address derivation and witness unlocking:

```bash
cargo test --test lab_04 -- --nocapture
```

## Terminal output

The public tests passed with 4 successful checks:

```text
running 4 tests
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok
test leaves_scriptsig_empty_and_uses_witness ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Observed test artifacts:
- Native Regtest Address: `bcrt1qvh0e9p3k0949dsq4q4590xsq4c76g8w7x4d2h2`
- Witness Version: `0`
- Witness Program: `80e461019056d683057e937d975db353c82d499a` (20 bytes)
- Locking Script (scriptPubKey): `001480e461019056d683057e937d975db353c82d499a`
- ScriptSig: empty (`""`)
- Witness Stack: `["30440220cafebabe01", "02e6642fd69bd211f93f7f1f36ec5100b9cb603f5b1b69fec05653d4c6353f4020"]`

## Evidence references

- Test suite implementation: `tests/lab_04.rs`
- Source logic: `src/labs/lab04_p2wpkh.rs`
- Automated execution log: `grading/logs/lab_04.log`

## Explanation

Native SegWit (P2WPKH, BIP141) fundamentally redesigns how transaction signatures and public keys are transported and verified on the Bitcoin network:

1. Empty ScriptSig:
   In native P2WPKH, the scriptSig field in the transaction input is left completely empty (zero bytes). Legacy nodes perceive a scriptPubKey of `OP_0 <20-byte-hash>` as an anyone-can-spend output and succeed immediately because pushing a non-zero element onto the stack leaves a truthy top element.

2. Witness Stack Placement:
   Upgraded SegWit nodes inspect the dedicated witness stack associated with the input. The signature and public key are supplied as separate witness items rather than serialized byte pushes in scriptSig.

3. Contrast with Legacy and Wrapped SegWit:
   - In Legacy P2PKH, unlocking data is pushed inside scriptSig, counting toward standard byte size and causing transaction malleability because scriptSig is part of the legacy TXID double-SHA256 calculation.
   - In P2SH-P2WPKH (wrapped), scriptSig is not empty; it contains a push of the 22-byte redeemScript (`0014<hash>`).
   - In Native P2WPKH, scriptSig is 0 bytes, eliminating malleability from the TXID hash and benefiting from the BIP141 75% witness discount (1 witness byte = 1 weight unit instead of 4).
