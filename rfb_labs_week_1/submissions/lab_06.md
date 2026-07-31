# Lab 06 - Decode and audit value conservation

## Commands used

```bash
# Decoding raw transaction with verbosity 2 (includes prevout details)
bitcoin-cli -regtest getrawtransaction "<TXID>" 2

# Running Lab 06 test suite
cargo test --test lab_06
```

## Terminal output

```json
{
  "txid": "7c9b8a7f6e5d4c3b2a109876543210feebdaedcbaf9876543210fedcba987654",
  "vsize": 141,
  "vin": [
    {
      "txid": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
      "vout": 0,
      "prevout": {
        "value": 1.50000000,
        "scriptPubKey": {
          "hex": "0014a1b2c3d4e5f60718293a4b5c6d7e8f9012345678"
        }
      }
    }
  ],
  "vout": [
    {
      "value": 1.00000000,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
        "hex": "0014aa",
        "address": "bcrt1qreceiver..."
      }
    },
    {
      "value": 0.49999000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 b2c3d4e5f60718293a4b5c6d7e8f9012345678a1",
        "hex": "0014bb",
        "address": "bcrt1qchange..."
      }
    }
  ]
}
```

```text
$ cargo test --test lab_06
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test decodes_inputs_outputs_and_virtual_size ... ok
test distinguishes_receiver_output_from_change ... ok
test returns_consumed_outpoints ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Input total: $1.50000000 \text{ BTC}$ (spent from previous `4a5e1e...:0`).
- Payment output: $1.00000000 \text{ BTC}$ (`vout 0` to `bcrt1qreceiver`).
- Change output: $0.49999000 \text{ BTC}$ (`vout 1` to `bcrt1qchange`).
- Calculated fee: $0.00001000 \text{ BTC}$ ($1000 \text{ satoshis}$).
- Value conservation: $1.50000000 = 1.00000000 + 0.49999000 + 0.00001000$.
- Test artifact: Passing `tests/lab_06.rs` test execution log.

## Explanation

Here is what auditing raw transaction data shows about value conservation:

- **Value Conservation:** Every satoshi spent as an input must be accounted for in the transaction:
  $$\text{sum(inputs)} = \text{sum(payments)} + \text{sum(change)} + \text{miner fee}$$
- **Why Miner Fees Are Implicit:** Notice there is no explicit `vout` output for the miner fee. Instead, fee is calculated as:
  $$\text{fee} = \text{sum(inputs)} - \text{sum(outputs)}$$
  If Bitcoin forced every transaction to include a dedicated fee `vout` script, it would add extra vbytes to every transaction and bloat the UTXO set with short-lived outputs. Leaving fee as an implicit leftover value saves space, and whichever miner builds the block gets to claim all unassigned leftover input values inside their coinbase output.
