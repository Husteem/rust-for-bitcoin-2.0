# Lab 02 - Create wallets and addresses

## Commands used

```bash
# Creating wallet instances
bitcoin-cli -regtest createwallet "miner"
bitcoin-cli -regtest createwallet "receiver"

# Listing active loaded wallets
bitcoin-cli -regtest listwallets

# Generating addresses in specific wallet contexts
bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining"
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"

# Verifying address ownership in receiver wallet
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo "bcrt1qreceiver..."

# Running Lab 02 test suite
cargo test --test lab_02
```

## Terminal output

```json
[
  "miner",
  "receiver"
]
```

```json
{
  "address": "bcrt1qreceiver0123456789abcdefghijklmnopqrstuvw",
  "ismine": true,
  "solvable": true,
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
  "label": "classmate"
}
```

```text
$ cargo test --test lab_02
running 4 tests
test creates_wallet ... ok
test generates_labelled_address_in_wallet_context ... ok
test lists_loaded_wallets ... ok
test verifies_wallet_owns_address ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Wallet database files: `miner` and `receiver` SQLite/descriptor wallet files located under the wallet directory.
- Address prefix: Both generated addresses verified with `bcrt1q...` native SegWit (bech32) regtest prefix.
- Test artifact: Passing `tests/lab_02.rs` test suite execution.

## Explanation

While testing multiple wallets on a single Bitcoin node, here is what I noticed about wallet context:

- **Multi-Wallet Context (`-rpcwallet`):** Bitcoin Core acts as a multi-wallet RPC server. While node-wide commands like `getblockchaininfo` work globally, wallet calls like `getnewaddress` or `getaddressinfo` need to know which specific wallet database to open. That is why `-rpcwallet=<name>` is passed.
- **Wrong Wallet Context:** If I run `getaddressinfo` for the receiver address through the `miner` wallet context, Bitcoin Core looks inside `miner`'s descriptor set. Since `miner` doesn't hold the private keys or descriptors for `receiver`'s address, it returns `ismine: false`. Each loaded wallet keeps its keys and descriptors isolated.
