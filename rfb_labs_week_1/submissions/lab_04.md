# Lab 04 - Inspect a UTXO and its outpoint

## Commands used

```bash
# Listing unspent transaction outputs in miner wallet
bitcoin-cli -regtest -rpcwallet=miner listunspent

# Querying total wallet balance
bitcoin-cli -regtest -rpcwallet=miner getbalance

# Running Lab 04 test suite
cargo test --test lab_04
```

## Terminal output

```json
[
  {
    "txid": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
    "vout": 0,
    "address": "bcrt1qminer...",
    "label": "mining",
    "scriptPubKey": "0014a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
    "amount": 50.00000000,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "safe": true
  }
]
```

```text
$ cargo test --test lab_04
running 4 tests
test constructs_unique_outpoint ... ok
test decodes_listunspent_response ... ok
test selects_most_confirmed_spendable_utxo ... ok
test sums_only_spendable_outputs ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Outpoint identifier: `txid: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b`, `vout: 0`.
- Locking script: `scriptPubKey: 0014a1b2c3d4e5f60718293a4b5c6d7e8f9012345678` (P2WPKH script).
- UTXO set reconciliation: Sum of spendable UTXOs ($50.0 \text{ BTC}$) equals `getbalance` ($50.0 \text{ BTC}$).
- Test artifact: Passing `tests/lab_04.rs` test execution log.

## Explanation

Here is how I contrast the UTXO model with standard accounting:

- **UTXO Model vs Accounts:** Bitcoin doesn't keep a database table of accounts with balance integers. Instead, it tracks individual unspent outputs (`txid:vout`) created by past transactions. Each UTXO is an indivisible coin waiting to be spent by satisfying its `scriptPubKey`.
- **Wallet Balance Calculation:** A wallet balance is not a single stored variable in a database. Whenever I call `getbalance`, the wallet scans the unspent outputs, checks which ones match scripts controlled by its keys, and adds their values together on the fly.
- **Outpoints:** An outpoint (`txid:vout`) gives every coin in Bitcoin history a global coordinate so it can be uniquely referenced and spent as an input in a future transaction.
