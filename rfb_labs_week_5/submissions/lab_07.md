# Lab 07 — BIP39 mnemonic and seed

## Commands used

I tested BIP39 mnemonic validation, entropy/checksum calculations, and PBKDF2 seed generation using the published class test mnemonic:

```bash
cargo test --test lab_07 -- --nocapture
```

## Terminal output

All 4 test cases passed cleanly:

```text
running 4 tests
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

Observed test parameters:
- Mnemonic: `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
- Word count: `12`
- Initial Entropy (ENT): `128 bits`
- Checksum length (CS): `4 bits` (total: 132 bits = 12 words * 11 bits)
- BIP39 Seed with passphrase `"TREZOR"`: `c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04`

## Evidence references

- Test suite implementation: `tests/lab_07.rs`
- Source logic: `src/labs/lab07_bip39.rs`
- Automated execution log: `grading/logs/lab_07.log`

## Explanation

The BIP39 key generation pipeline follows a sequence of deterministic transformations:

1. Entropy (ENT):
   The true cryptographic randomness (128 bits for 12 words, up to 256 bits for 24 words).

2. Checksum (CS):
   Computed as the first ENT / 32 bits of SHA256(ENT). The checksum is appended to the entropy to detect transcription errors, typos, or word transpositions during wallet recovery. It provides error detection, not encryption.

3. Mnemonic Sentence:
   The combined ENT + CS bitstring is sliced into 11-bit chunks, each indexing a standardized 2048-word dictionary.

4. Seed Derivation (PBKDF2):
   The mnemonic sentence (normalized with UTF-8 NFKD) is passed to PBKDF2-HMAC-SHA512 using 2048 iterations, with salt format!("mnemonic{}", passphrase). This yields a 512-bit binary seed.

5. Role of the Optional Passphrase:
   The passphrase acts as an extra salt component. Any alteration in the passphrase creates an entirely distinct 512-bit seed (and thus an entirely different wallet). Because PBKDF2 is a cryptographic one-way key derivation function, a forgotten passphrase cannot be mathematically derived or recovered from the mnemonic alone.
