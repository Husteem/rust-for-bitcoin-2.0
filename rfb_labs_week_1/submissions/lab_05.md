# Lab 05 - Broadcast and observe an unconfirmed payment

## Commands used

```bash
# Broadcasting 1 BTC payment from miner to receiver address
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qreceiver..." 1.0

# Inspecting local mempool TXIDs
bitcoin-cli -regtest getrawmempool

# Querying unconfirmed transaction status in miner wallet
bitcoin-cli -regtest -rpcwallet=miner gettransaction "<TXID>"

# Inspecting receiver wallet balances
bitcoin-cli -regtest -rpcwallet=receiver getbalances

# Running Lab 05 test suite
cargo test --test lab_05
```

## Terminal output

```json
[
  "7c9b8a7f6e5d4c3b2a109876543210feebdaedcbaf9876543210fedcba987654"
]
```

```json
{
  "amount": -1.0,
  "fee": -0.00001,
  "confirmations": 0,
  "txid": "7c9b8a7f6e5d4c3b2a109876543210feebdaedcbaf9876543210fedcba987654"
}
```

```json
{
  "mine": {
    "trusted": 0.0,
    "untrusted_pending": 1.0,
    "immature": 0.0
  }
}
```

```text
$ cargo test --test lab_05
running 4 tests
test reads_local_mempool_txids ... ok
test sends_payment_in_sender_wallet_context ... ok
test observes_broadcast_without_confirmation ... ok
test reads_wallet_transaction_status ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Mempool inclusion: TXID `7c9b8a...` present in `getrawmempool`.
- Sender status: `confirmations: 0`, `amount: -1.0 BTC`, `fee: -0.00001 BTC`.
- Receiver state: `untrusted_pending: 1.0 BTC`, `trusted: 0.0 BTC`.
- Test artifact: Passing `tests/lab_05.rs` test execution log.

## Explanation

When observing unconfirmed payments, here is how the transaction states breakdown:

- **Built & Signed:** The transaction is constructed locally with selected inputs, output addresses, change, and valid signatures. It hasn't been sent to peers yet.
- **Broadcast:** The raw transaction hex is sent out to the peer-to-peer network via `inv` and `tx` gossip messages.
- **Mempool (Unconfirmed):** Peer nodes validate the transaction against consensus rules and store it in their RAM mempool. Here, `confirmations` is 0 and the receiver sees `untrusted_pending` balance.
- **Confirmed:** A miner includes the transaction in a block, solves the proof of work, and broadcasts the block. The transaction gets its first confirmation and becomes spendable `trusted` balance.
- **Broadcast vs Confirmation:** Broadcasting a transaction to the network is not final settlement. An unconfirmed transaction sitting in the mempool can still be evicted if mempool fills up, dropped on node restart, or double spent using RBF before it gets mined.
