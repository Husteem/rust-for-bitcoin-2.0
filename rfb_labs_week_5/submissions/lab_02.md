# Lab 02 — Legacy P2PKH

## Commands used

I ran the test suite for legacy P2PKH construction and unlocking verification:

```bash
cargo test --test lab_02 -- --nocapture
```

## Terminal output

All 4 test cases passed successfully:

```text
running 4 tests
test builds_the_standard_p2pkh_lock ... ok
test commits_to_hash160_of_the_public_key ... ok
test derives_the_expected_p2pkh_address ... ok
test puts_unlocking_data_in_scriptsig ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Observed test artifacts:
- Compressed Public Key: `02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5`
- Derived P2PKH Address (Mainnet): `16UwLL9Risc3QfPqBUvKofHmBQ7wMtjvM`
- Committed Public Key Hash (HASH160): `06226e46111a0b59caaf126043eb5bbf28c34f3a`
- Constructed scriptPubKey: `76a91406226e46111a0b59caaf126043eb5bbf28c34f3a88ac`

## Evidence references

- Test suite implementation: `tests/lab_02.rs`
- Source logic: `src/labs/lab02_p2pkh.rs`
- Automated execution log: `grading/logs/lab_02.log`

## Explanation

P2PKH (Pay-to-Public-Key-Hash) operates on a two-step verification model separating key identity commitment from spend authorization:

1. Locking Script (scriptPubKey):
   The locking script `OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG` commits only to the 20-byte HASH160 (RIPEMD160 of SHA256) of the recipient public key. This keeps the on-chain UTXO footprint compact and shields the full public key from quantum or cryptographic exposure until the output is spent.

2. Unlocking Script (scriptSig):
   To spend the UTXO, the spender must provide `<signature> <pubKey>` in the legacy scriptSig field (with an empty witness stack). 

3. Stack Execution:
   During script evaluation, `<signature>` and `<pubKey>` are pushed onto the execution stack. `OP_DUP` duplicates the public key, `OP_HASH160` computes its hash, and `OP_EQUALVERIFY` asserts that this hash matches the committed `<pubKeyHash>`. Finally, `OP_CHECKSIG` validates the ECDSA signature against the duplicated public key and the transaction hash preimage.
