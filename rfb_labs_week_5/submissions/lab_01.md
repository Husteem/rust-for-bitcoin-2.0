# Lab 01 — Address and network identification

## Commands used

I executed the dedicated test suite for address inspection and network validation:

```bash
cargo test --test lab_01 -- --nocapture
cargo clippy --all-targets --all-features -- -D warnings
```

## Terminal output

The public test suite verified prefix mapping, address inspection, scriptPubKey generation, and network rejection:

```text
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Sample address report inspected during execution:
- Address: `mrCDrCybB6J1vRfbwM5hemdJz73FwDBC8r`
- Network: `regtest`
- Format: `P2pkh`
- scriptPubKey: `76a914751e76e8199196d454941c45d1b3a323f1433bd688ac`

## Evidence references

- Test suite implementation: `tests/lab_01.rs`
- Source logic: `src/labs/lab01_addresses.rs`
- Automated execution log: `grading/logs/lab_01.log`

## Explanation

Prefix inspection is only a preliminary heuristic based on human-readable string conventions. While prefixes such as '1', '3', 'bc1q', or 'bcrt1q' give an immediate visual clue regarding the intended address family and network, prefix inspection alone does not guarantee validity.

Full validation requires:
1. Checksum verification: Base58Check uses double-SHA256 checksumming, while Bech32 and Bech32m use BCH error-detecting polynomials. A string may possess the correct prefix but contain typos, invalid characters, or bit flips that only full checksum evaluation will detect.
2. Network enforcement: The payload must match the active consensus network parameters (such as mainnet versus testnet/regtest HRP or version bytes). Without network validation, transactions risk sending funds across incompatible network chains.
3. Payload and script integrity: The decoded payload must match strict length and format rules, such as 20 bytes for HASH160/P2WPKH or 32 bytes for P2TR.
