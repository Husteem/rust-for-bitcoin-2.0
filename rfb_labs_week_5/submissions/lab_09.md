# Lab 09 — BIP44 path decoding

## Commands used

I executed the test suite for BIP44 derivation path parsing, level explanation, and child address generation:

```bash
cargo test --test lab_09 -- --nocapture
```

## Terminal output

All 4 test cases passed:

```text
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test derives_the_selected_bip44_address ... ok
test explains_zero_based_account_and_chain ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Observed test output:
- Decoded Path `m/44'/0'/2'/1/5`:
  - Purpose: `44'` (hardened)
  - Coin Type: `0'` (hardened, Bitcoin Mainnet)
  - Account: `2'` (hardened, third account)
  - Change: `1` (unhardened, internal / change chain)
  - Address Index: `5` (unhardened, sixth address in chain)
- Modified Address Index (`5` -> `6`): `m/44'/0'/2'/1/6`
- Derived BIP44 Regtest Address (`m/44'/1'/0'/0/0`): `mkwD5HqriH7hPzqcYCiNDPsmupnChrvjyR`

## Evidence references

- Test suite implementation: `tests/lab_09.rs`
- Source logic: `src/labs/lab09_bip44.rs`
- Automated execution log: `grading/logs/lab_09.log`

## Explanation

BIP44 establishes a standardized five-level derivation path hierarchy:
m / purpose' / coin_type' / account' / change / address_index

1. Purpose (`44'`):
   A hardened constant indicating compliance with the BIP44 specification (legacy P2PKH).

2. Coin Type (`0'` for Bitcoin Mainnet, `1'` for Testnet/Regtest):
   A hardened constant assigned by SLIP-0044 ensuring separate subtrees for different cryptocurrencies and test networks.

3. Account (`account'`):
   A hardened zero-based index allowing users to compartmentalize funds (e.g. index 0 is the first account, index 1 is the second account, index 2 is the third account). Hardening at this level ensures that exporting an account-level xpub does not compromise other accounts.

4. Change (`0` for external / receive, `1` for internal / change):
   An unhardened index dividing addresses into public receive addresses versus private change addresses used when spending change back to oneself.

5. Address Index (`address_index`):
   An unhardened zero-based sequential index generating individual receiving or change addresses (e.g. index 0 is the first address, index 5 is the sixth address).
