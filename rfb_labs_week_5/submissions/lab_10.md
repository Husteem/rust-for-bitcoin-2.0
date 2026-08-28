# Lab 10 — Deterministic recovery across address families

## Commands used

I tested deterministic recovery across BIP44, BIP49, and BIP84 standard hierarchies using the test mnemonic and passphrase:

```bash
cargo test --test lab_10 -- --nocapture
```

## Terminal output

All 4 test cases passed:

```text
running 4 tests
test changing_only_the_index_changes_the_address ... ok
test derives_three_regtest_address_families ... ok
test format_selection_changes_the_lock_target ... ok
test identical_recovery_inputs_repeat ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

Observed derived addresses on Regtest (Account `0`, Index `0`):
- BIP44 (P2PKH, `m/44'/1'/0'/0/0`): `mkwD5HqriH7hPzqcYCiNDPsmupnChrvjyR`
- BIP49 (P2SH-P2WPKH, `m/49'/1'/0'/0/0`): `2N2wS5tL21w4Qv3p1C6z6Z4iQ8zM5bYjL7G`
- BIP84 (Native P2WPKH, `m/84'/1'/0'/0/0`): `bcrt1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`
- Repeatability check: identical inputs deterministically recreate the exact same address string (`true`).
- Index progression check: changing index from 0 to 1 produces a distinct address (`true`).

## Evidence references

- Test suite implementation: `tests/lab_10.rs`
- Source logic: `src/labs/lab10_recovery.rs`
- Automated execution log: `grading/logs/lab_10.log`

## Explanation

Deterministic wallet recovery relies on mathematical algorithms combined with standard derivation and script conventions:

1. How Inputs Reproduce Keys:
   Given the identical mnemonic phrase and optional passphrase, BIP39 PBKDF2 produces the exact same 512-bit binary seed. Starting from this seed, BIP32 HMAC-SHA512 functions iteratively evaluate each step along a derivation path, yielding the exact same private key, public key, and chain code every time.

2. Why Path and Script Conventions are Required:
   - A single master seed can generate trillions of valid keypairs across infinite possible derivation paths.
   - Without agreed-upon conventions (BIP44 for legacy P2PKH, BIP49 for wrapped P2WPKH, and BIP84 for native P2WPKH), a wallet recovering a seed would have no idea which paths to scan or how to format the public keys into addresses.
   - For instance, deriving a key at `m/84'/0'/0'/0/0` and constructing a P2PKH script produces a completely different address and UTXO lock than constructing a native P2WPKH witness program with the exact same key. Successful wallet restoration requires both the underlying recovery material and standard path/script conventions.
