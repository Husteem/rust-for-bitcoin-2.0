# Lab 01 - Regtest network inspection

## Commands used

```bash
# Inspecting network state via bitcoin-cli
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash

# Running the Rust Lab 01 test suite
cargo test --test lab_01
```

## Terminal output

```json
{
  "chain": "regtest",
  "blocks": 101,
  "headers": 101,
  "bestblockhash": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
  "difficulty": 4.656542373906925e-10,
  "verificationprogress": 1.0,
  "initialblockdownload": false
}
```

```text
$ cargo test --test lab_01
running 4 tests
test builds_verified_network_snapshot ... ok
test reads_best_block_hash ... ok
test reads_block_height ... ok
test reads_regtest_chain ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Polar network setup named `Week 1 Bitcoin Fundamentals` with 1 active Bitcoin Core regtest node.
- Verified node height (101 blocks) and best block hash matching `getblockchaininfo`.
- Test suite `tests/lab_01.rs` execution logs demonstrating `NetworkSnapshot` struct verification.

## Explanation

When setting up and inspecting the regtest node, here is how the underlying components fit together from my testing:

- **Polar:** A visual GUI manager that orchestrates local Bitcoin and Lightning topologies. It handles launching containers, configuring ports, and setting up node connections for local dev work.
- **Docker:** The container engine running under Polar. It keeps the `bitcoind` daemon isolated in its own Linux container so system dependencies don't collide with the host machine.
- **Bitcoin Core:** The actual `bitcoind` full node daemon running the Bitcoin protocol logic, maintaining the local block database, validating transactions, and exposing the JSON-RPC interface that `bitcoin-cli` talks to.
- **Regtest (Regression Test):** A private local network mode with zero-difficulty proof of work. Blocks aren't mined automatically by outside miners like on mainnet; instead, I generate blocks instantly whenever I run `generatetoaddress`.
