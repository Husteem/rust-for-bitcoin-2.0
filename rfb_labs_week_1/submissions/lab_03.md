# Lab 03 - Demonstrate coinbase maturity

## Commands used

```bash
# Mining initial block to miner address
bitcoin-cli -regtest generatetoaddress 1 "bcrt1qminer..."

# Inspecting wallet balances after 1 block
bitcoin-cli -regtest -rpcwallet=miner getbalances

# Attempting premature spend of immature coinbase reward (fails)
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qreceiver..." 1.0

# Mining 100 additional blocks to reach coinbase maturity depth
bitcoin-cli -regtest generatetoaddress 100 "bcrt1qminer..."

# Inspecting wallet balances after 101 total blocks
bitcoin-cli -regtest -rpcwallet=miner getbalances

# Running Lab 03 test suite
cargo test --test lab_03
```

## Terminal output

```json
{
  "mine": {
    "trusted": 0.0,
    "untrusted_pending": 0.0,
    "immature": 50.0
  }
}
```

```text
error code: -6
error message: Insufficient funds
```

```json
{
  "mine": {
    "trusted": 50.0,
    "untrusted_pending": 0.0,
    "immature": 5000.0
  }
}
```

```text
$ cargo test --test lab_03
running 4 tests
test demonstrates_first_coinbase_becoming_spendable_at_height_101 ... ok
test mines_requested_number_of_blocks ... ok
test preserves_insufficient_funds_error ... ok
test reads_nested_wallet_balances ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Initial state at height 1: `trusted: 0.0 BTC`, `immature: 50.0 BTC`.
- Premature spend error: Captured RPC error code `-6` (`Insufficient funds`) when trying to spend unconfirmed coinbase value.
- Mature state at height 101: `trusted: 50.0 BTC`, `immature: 5000.0 BTC`.
- Test artifact: Passing `tests/lab_03.rs` test execution log.

## Explanation

Here is how I understand coinbase maturity after walking through this test step by step:

- **`COINBASE_MATURITY = 100` Rule:** Bitcoin consensus rules prevent newly mined block rewards (coinbase outputs) from being spent until 100 additional blocks are mined on top of the block where they were generated. This protects the network during chain reorganizations. If a miner could spend a block subsidy immediately and that block got orphaned 2 blocks later, those coins would disappear, invalidating every downstream transaction built on them.
- **Why 101 Blocks on a Fresh Chain:** On a brand new regtest chain, block 1 generates the first 50 BTC reward. Mining 100 more blocks takes the chain height to 101. At height 101, the block 1 reward has 100 confirmations built on top of it ($101 - 1 = 100$). That satisfies the maturity requirement, turning the first 50 BTC into spendable `trusted` balance while the rewards from blocks 2 to 101 remain `immature`.
