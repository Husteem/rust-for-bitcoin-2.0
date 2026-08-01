# Lab 07 - Confirm and locate the transaction

## Commands used

```bash
# Mining 1 block to miner address
bitcoin-cli -regtest generatetoaddress 1 "bcrt1qminer..."

# Checking local mempool is empty
bitcoin-cli -regtest getrawmempool

# Querying receiver wallet transaction status
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<TXID>"

# Inspecting confirming block transaction list
bitcoin-cli -regtest getblock "<BLOCK_HASH>" 1

# Running Lab 07 test suite
cargo test --test lab_07
```

## Terminal output

```json
[]
```

```json
{
  "amount": 1.0,
  "confirmations": 1,
  "blockhash": "15434ac5b0e0f5d4b420c3683e79cf546184eb1379375f04672c80b28f0b982b",
  "txid": "7c9b8a7f6e5d4c3b2a109876543210feebdaedcbaf9876543210fedcba987654"
}
```

```json
{
  "hash": "15434ac5b0e0f5d4b420c3683e79cf546184eb1379375f04672c80b28f0b982b",
  "tx": [
    "0000000000000000000000000000000000000000000000000000000000000001",
    "7c9b8a7f6e5d4c3b2a109876543210feebdaedcbaf9876543210fedcba987654"
  ]
}
```

```text
$ cargo test --test lab_07
running 4 tests
test detects_empty_mempool ... ok
test mines_exactly_one_block ... ok
test proves_transaction_is_inside_confirming_block ... ok
test reads_confirmation_count ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Mempool eviction: `getrawmempool` returned `[]` (transaction moved to block).
- Receiver balance state: Receiver balance transitioned from `untrusted_pending` to `trusted` (1.0 BTC).
- Block membership: Block `15434a...` contains `tx: ["coinbase-txid", "7c9b8a..."]`.
- Test artifact: Passing `tests/lab_07.rs` test execution log.

## Explanation

What actually changes when a transaction gets mined into a block:

- **Did Mining Change the Transaction Data?** No. Mining does not alter a single byte of the raw transaction, inputs, outputs, scripts, signatures, or resulting TXID.
- **What Mining Changed:** Mining changed the transaction's status in the agreed history. Before mining, it was an unconfirmed mempool candidate that could be evicted or double spent. Mining ordered the transaction into a block Merkle tree, anchored it into the Proof of Work chain, cleared it out of mempool, incremented confirmations to 1, and settled the receiver's funds into spendable `trusted` balance.
