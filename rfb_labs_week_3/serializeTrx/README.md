# Bitcoin Transaction Serializer CLI

A command-line tool written in Rust to serialize Bitcoin transactions dynamically based on user-provided arguments. It supports both legacy and SegWit inputs, outputs, witness data, and locktime.

## Features

- Dynamic CLI transaction configuration (no hardcoded transaction data)
- Multi-input and multi-output transaction generation
- Support for SegWit marker, flag, and witness items per input
- Comprehensive input formatting validations (even hex strings, valid integers, and correct txid sizes)

## How to Build

From the root of the workspace or package directory:

```bash
cargo build --release
```

## How to Run

Provide inputs and outputs as structured arguments.

### Arguments

- `--version <INT>`: The transaction version (defaults to `2`).
- `--segwit`: Set this flag to compile a SegWit serialization.
- `--input <prev_txid>:<vout>:<sequence>[:script_sig_hex]`: Can be supplied multiple times.
- `--output <value_sats>:<script_pubkey_hex>`: Can be supplied multiple times.
- `--witness <input_idx>:<witness_item_hex>`: Can be supplied multiple times to associate witness stacks with specific inputs.
- `--locktime <INT>`: Locktime field (defaults to `0`).

---

## Examples

### 1. SegWit Transaction (1 Input, 2 Outputs, 2 Witness items)

```bash
cargo run --bin serialize_trx -- \
  --segwit \
  --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295 \
  --witness 0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301 \
  --witness 0:029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358 \
  --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687
```

**Output:**
```
Serialized transaction:
[2, 0, 0, 0, 0, 1, 1, 143, 176, ... 0, 0, 0, 0]
Serialized Hex transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

### 2. Legacy Transaction (1 Input, 1 Output, No Witness)

```bash
cargo run --bin serialize_trx -- \
  --version 1 \
  --input 7b1eabe0209b1fe794124575ef807057c77ada2138ae4fa8d6c4de0398a14f3f:0:4294967295 \
  --output 1000000:76a9142e00b21a8d052601ee34d0b7a8d0032e3a1f9d5f88ac
```
