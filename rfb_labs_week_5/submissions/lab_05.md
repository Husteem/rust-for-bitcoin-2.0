# Lab 05 — Address compatibility map

## Commands used

I ran the test suite evaluating sender compatibility matrices across address encodings:

```bash
cargo test --test lab_05 -- --nocapture
```

## Terminal output

All 4 test assertions passed:

```text
running 4 tests
test builds_the_four_format_map ... ok
test names_the_required_human_encoding ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Observed compatibility mapping for a P2SH-era wallet:
- `p2pkh`: `true` (Base58Check)
- `p2sh_p2wpkh`: `true` (Base58Check)
- `p2wpkh`: `false` (Requires Bech32)
- `p2tr`: `false` (Requires Bech32m)

## Evidence references

- Test suite implementation: `tests/lab_05.rs`
- Source logic: `src/labs/lab05_compatibility.rs`
- Automated execution log: `grading/logs/lab_05.log`

## Explanation

Address compatibility between sender wallets and recipient address formats is determined by the sender's ability to parse and encode the destination string into an on-chain scriptPubKey:

1. Why a P2SH-Era Wallet Accepts `3...` but Rejects `bc1q...`:
   - A legacy P2SH-era wallet understands Base58Check encoding and knows how to decode address strings starting with `1` (P2PKH) and `3` (P2SH). When given a `3...` address representing P2SH-wrapped SegWit (P2SH-P2WPKH), the wallet simply decodes the 20-byte hash and constructs a standard `OP_HASH160 <hash> OP_EQUAL` output. The sender wallet does not even need to know SegWit exists.
   - When presented with a `bc1q...` address, an older wallet fails at string parsing because it does not implement the Bech32 character set, checksum algorithm, or SegWit version rules, rejecting the address as invalid input.

2. Sending Support vs Spending Support:
   - Sending support only requires decoding an address string and creating the corresponding scriptPubKey in a transaction output.
   - Spending support requires managing private keys, constructing the proper unlocking scriptSig or witness stack, signing the appropriate sighash algorithm (e.g. BIP143 for SegWit, BIP341 for Taproot), and computing witness serialization.
