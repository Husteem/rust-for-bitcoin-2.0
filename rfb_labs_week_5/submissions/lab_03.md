# Lab 03 — P2SH 2-of-3 multisig

## Commands used

I executed the test suite for wrapping 2-of-3 multisig inside P2SH:

```bash
cargo test --test lab_03 -- --nocapture
```

## Terminal output

The tests confirmed redeem script generation, outer address derivation, and multi-layer script reporting:

```text
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test builds_a_two_of_three_redeem_script ... ok
test derives_the_committed_p2sh_address ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Observed test artifacts:
- Inner RedeemScript: `52210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817982102c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee52102f9308a01f78c14f682d13f62a3c562904d445802a37f3a2d4263b82dcc523e5953ae`
- P2SH Regtest Address: `2N8hwP1WmJrFF5QWABn38y63uYLhnJYJYTF`
- Outer scriptPubKey: `a914a5d85c4f0a23e2f6714ecd81559b4071fa0935f487`

## Evidence references

- Test suite implementation: `tests/lab_03.rs`
- Source logic: `src/labs/lab03_p2sh.rs`
- Automated execution log: `grading/logs/lab_03.log`

## Explanation

P2SH (Pay-to-Script-Hash, defined in BIP16) wraps complex spending scripts behind a simple 20-byte hash commitment, executing evaluation in two distinct stages:

1. Outer Hash Check:
   The UTXO scriptPubKey is `OP_HASH160 <scriptHash> OP_EQUAL`. The spender supplies `<sig1> <sig2> ... <redeemScript>` in scriptSig. First, the script execution engine computes `HASH160(redeemScript)` and compares it against `<scriptHash>`. If they do not match, validation fails immediately.

2. Inner Multisig Check:
   Passing the outer hash check proves that the spender provided the preimage script committed to by the sender. However, this alone does not authorize spending. The Bitcoin interpreter then deserializes the `<redeemScript>` bytes and executes them as script instructions using the remaining stack items (`<sig1> <sig2>`). For a 2-of-3 multisig, `OP_CHECKMULTISIG` executes to verify that the provided signatures are valid under at least 2 of the 3 specified public keys.
