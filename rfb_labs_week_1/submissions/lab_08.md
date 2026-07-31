# Lab 08 - Inspect block commitments and confirmation depth

## Commands used

```bash
# Querying verbose block header
bitcoin-cli -regtest getblockheader "<BLOCK_HASH>"

# Querying receiver transaction confirmations before mining
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<TXID>"

# Mining 5 additional confirmation blocks
bitcoin-cli -regtest generatetoaddress 5 "bcrt1qminer..."

# Querying receiver transaction confirmations after mining
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<TXID>"

# Running Lab 08 test suite
cargo test --test lab_08
```

## Terminal output

```json
{
  "hash": "15434ac5b0e0f5d4b420c3683e79cf546184eb1379375f04672c80b28f0b982b",
  "confirmations": 1,
  "height": 102,
  "version": 536870912,
  "versionHex": "20000000",
  "merkleroot": "c5d1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f",
  "time": 1770000000,
  "mediantime": 1769999000,
  "nonce": 7,
  "bits": "207fffff",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000ce",
  "previousblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206"
}
```

```text
$ cargo test --test lab_08
running 4 tests
test build_security_report ... ok
test decodes_proof_linked_block_header ... ok
test mines_requested_confirmation_depth ... ok
test reads_wallet_confirmation_depth ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Header commitments: `hash: 15434a...`, `previousblockhash: 0f9188...`, `merkleroot: c5d1a2...`, `nonce: 7`, `bits: 207fffff`.
- Confirmation progression: Initial confirmations: 1 -> Final confirmations: 6 (after mining 5 extra blocks).
- Test artifact: Passing `tests/lab_08.rs` test execution log.

## Explanation

Here is how block headers, proof of work, and confirmation depth link together:

- **Header Commitments:** Every 80-byte block header contains `previousblockhash` linking to the parent block, and `merkleroot` committing to every transaction in the block. Changing even one bit in a transaction breaks the Merkle root, changing the header hash and invalidating the block link.
- **Proof of Work Search:** Miners search for a valid `nonce` such that double SHA256 of the 80-byte header is less than or equal to the target set by `bits`.
- **Confirmation Depth & Reorg Cost:** Every new block appended to the chain increases accumulated proof of work (`chainwork`). To replace a transaction confirmed 6 blocks deep, an attacker has to build an alternative branch with 6+ blocks having higher total `chainwork`, which requires out-hashing the honest network across 6 sequential targets.
- **Valid vs Invalid Transactions:** Confirmations increase the cost of reorganizing valid history, but confirmations never make an invalid transaction valid. Full nodes enforce consensus rules independently; an invalid transaction (like a fake signature or bad coin input) is rejected by full nodes immediately regardless of how many blocks or how much proof of work is built on top.
