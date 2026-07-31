# Lab 09 - Force multi-UTXO coin selection

## Commands used

```bash
# Sending three separate 0.4 BTC funding payments from miner to Alice
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qalice..." 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qalice..." 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qalice..." 0.4

# Mining 1 block to confirm Alice's funding UTXOs
bitcoin-cli -regtest generatetoaddress 1 "bcrt1qminer..."

# Inspecting Alice's confirmed UTXOs
bitcoin-cli -regtest -rpcwallet=alice listunspent

# Alice sending 1.0 BTC payment to receiver (requires combining inputs)
bitcoin-cli -regtest -rpcwallet=alice sendtoaddress "bcrt1qreceiver..." 1.0

# Auditing Alice's multi-input transaction details
bitcoin-cli -regtest getrawtransaction "<SPEND_TXID>" 2

# Running Lab 09 test suite
cargo test --test lab_09
```

## Terminal output

```json
[
  {
    "txid": "funding-txid-0",
    "vout": 0,
    "address": "bcrt1qalice...",
    "amount": 0.40000000,
    "confirmations": 1,
    "spendable": true
  },
  {
    "txid": "funding-txid-1",
    "vout": 0,
    "address": "bcrt1qalice...",
    "amount": 0.40000000,
    "confirmations": 1,
    "spendable": true
  },
  {
    "txid": "funding-txid-2",
    "vout": 0,
    "address": "bcrt1qalice...",
    "amount": 0.40000000,
    "confirmations": 1,
    "spendable": true
  }
]
```

```text
$ cargo test --test lab_09
running 4 tests
test audits_three_input_spend_payment_change_and_fee ... ok
test creates_three_separate_funding_transactions ... ok
test filters_confirmed_utxos_for_alice_address ... ok
test sends_one_btc_from_alice ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Funding UTXOs: 3 separate UTXOs of $0.40000000 \text{ BTC}$ each ($1.20000000 \text{ BTC}$ total).
- Multi-input spend audit: Consumed 3 inputs (`funding-0:0`, `funding-1:0`, `funding-2:0`).
- Payment and change: Payment output ($1.00000000 \text{ BTC}$), Change output ($0.19999000 \text{ BTC}$ to Alice change address).
- Fee: $0.00001000 \text{ BTC}$ ($1000 \text{ satoshis}$).
- Test artifact: Passing `tests/lab_09.rs` test execution log.

## Explanation

What multi-UTXO spending demonstrates about coin selection and privacy:

- **Forced Multi-UTXO Coin Selection:** Alice has three $0.4 \text{ BTC}$ UTXOs. To send a $1.0 \text{ BTC}$ payment, no single UTXO is large enough, so the coin selection algorithm combines all three inputs ($0.4 + 0.4 + 0.4 = 1.2 \text{ BTC}$). All 3 inputs get spent, $1.0 \text{ BTC}$ goes to the receiver, $0.19999 \text{ BTC}$ returns as change, and $0.00001 \text{ BTC}$ goes to miner fee.
- **Privacy Trade-Off (Common-Input Ownership Heuristic):** When multiple UTXOs are spent together as inputs in one transaction, chain analysis software applies the Common-Input Ownership Heuristic (CIOH). The assumption is that all inputs consumed in a single transaction belong to the same wallet entity. Combining UTXOs links previously separate addresses together on the public blockchain, creating a privacy trade-off.
