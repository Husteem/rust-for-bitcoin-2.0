# Lab 08 — BIP32 extended keys

## Commands used

I tested BIP32 hierarchical deterministic key derivation, xpriv/xpub relationships, and hardened step detection using the disposable test mnemonic:

```bash
cargo test --test lab_08 -- --nocapture
```

## Terminal output

All 4 test cases passed:

```text
running 4 tests
test creates_a_test_family_master_xpriv ... ok
test derives_matching_extended_keys ... ok
test distinguishes_hardened_and_normal_paths ... ok
test xpub_derives_a_normal_public_child ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Observed key characteristics:
- Testnet/Regtest Master xpriv prefix: `tprv...`
- Derived Path `m/84'/1'/0'`: `tprv...` (xpriv) and `tpub...` (xpub)
- Child derivation on xpub (`index = 7`): derives matching `tpub...` without private key material.
- Hardened path detection: `m/44'/0'/0'/0/0` identified as containing hardened steps, `m/0/1/2` identified as unhardened.

## Evidence references

- Test suite implementation: `tests/lab_08.rs`
- Source logic: `src/labs/lab08_bip32.rs`
- Automated execution log: `grading/logs/lab_08.log`

## Explanation

BIP32 introduces Hierarchical Deterministic (HD) wallets, allowing a full tree of keypairs to be derived from a single root secret:

1. Extended Keys and Chain Code:
   An extended key consists of a 256-bit key (private or public) combined with a 256-bit entropy buffer called the chain code. The chain code prevents an observer who knows a parent key from guessing child keys without computing the HMAC-SHA512 step.

2. Normal (Unhardened) Derivation:
   - For index i < 2^31, child keys are derived by passing HMAC-SHA512(Parent ChainCode, Parent PublicKey || i).
   - Because only the parent public key is needed in the HMAC, an extended public key (xpub) can derive child public keys directly without private keys. This enables secure watch-only wallets and payment processors.

3. Hardened Derivation:
   - For index i >= 2^31 (indicated by i'), child keys are derived by passing HMAC-SHA512(Parent ChainCode, 0x00 || Parent PrivateKey || i).
   - Hardened derivation requires the parent private key. This creates a firewall: if a child private key is compromised, an attacker with the parent xpub cannot walk up the tree to deduce parent or sibling private keys. Consequently, hardened children cannot be derived from a parent xpub.
