# Week 6 Assignment: Bitcoin Wallet in Rust

A modular, persistent, command-line Bitcoin wallet written in Rust supporting both **BIP84 (Native SegWit P2WPKH)** and **BIP86 (Taproot P2TR)** with SQLite state persistence, multi-strategy coin selection, transaction building, and Bitcoin Core RPC integration.

---

## 1. Features & Architecture Overview

- **Key Management & Derivation**:
  - BIP39 standard 12-word mnemonic generation and passphrase-protected recovery.
  - BIP32 hierarchical deterministic key derivation.
  - BIP84 derivation (`m/84'/1'/0'/keychain/index`) for Native SegWit (`wpkh`).
  - BIP86 derivation (`m/86'/1'/0'/keychain/index`) for Taproot key-path spending (`tr`).
  - Output script descriptors with origin info: `wpkh([fingerprint/84'/1'/0']xpub/0/*)` and `tr([fingerprint/86'/1'/0']xpub/0/*)`.
- **Database Persistence (`rusqlite`)**:
  - Embedded SQLite database running in WAL mode.
  - State tracking for wallet metadata, derivation indices, derived addresses, unspent transaction outputs (UTXOs), and transaction history.
  - Survives application restarts with consistent balance queries.
- **Coin Selection & Fee Estimation**:
  - Flexible coin selection strategies: `LargestFirst`, `SmallestFirst`, and `ExactMatch`.
  - Accurate virtual size (`vsize`) and weight unit calculation for P2WPKH and P2TR inputs/outputs.
  - Dust threshold protection (546 satoshis): sub-dust change is folded into miner fees to prevent unspendable dust creation.
- **Transaction Construction & Signing**:
  - Unsigned transaction assembly with RBF signaling enabled.
  - SegWit v0 (BIP143) ECDSA signing with `SighashCache`.
  - Taproot (BIP341) Schnorr key-path signing with tweaked keypairs.
- **Bitcoin Core RPC Integration**:
  - RPC client connectivity with authentication via credentials or cookie file.
  - Fast UTXO scanning using Bitcoin Core's `scantxoutset` descriptor queries.
  - Raw transaction broadcasting via `sendrawtransaction`.
- **Raw `rust-bitcoin` Script Demonstration**:
  - Standalone low-level script builder constructing a hash-locked + signature-locked witness script (`OP_SHA256 <hash> OP_EQUALVERIFY <pubkey> OP_CHECKSIG`).
  - Direct BIP143 sighash calculation, witness stack assembly, and transaction serialization.

---

## 2. Project Layout

```
rfb_labs_week_6/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # Library module declarations
│   ├── main.rs             # CLI entrypoint and subcommand routing
│   ├── cli.rs              # Clap CLI definitions and arguments
│   ├── config.rs           # Network, descriptor, and RPC configuration types
│   ├── keys.rs             # BIP39 / BIP32 / BIP84 / BIP86 key and descriptor derivation
│   ├── db.rs               # SQLite schema and persistent storage operations
│   ├── coin_selection.rs   # Coin selection strategies and fee/vsize estimation
│   ├── tx.rs               # Transaction construction and SegWit/Taproot signing
│   ├── rpc.rs              # Bitcoin Core RPC client and UTXO scanner
│   └── raw_demo.rs         # Low-level script demo comparing raw scripts with descriptors
└── tests/
    ├── wallet_keys.rs      # Key generation, derivation path, and descriptor tests
    ├── wallet_db.rs        # SQLite state persistence and restart tests
    ├── coin_selection_test.rs # Coin selection and dust protection tests
    ├── tx_signing_test.rs  # SegWit and Taproot transaction signing tests
    └── raw_demo_test.rs    # Raw script execution test
```

---

## 3. Installation & Building

Compile the project and run all tests using standard Cargo commands:

```bash
# Check code syntax and types
cargo check

# Run all unit and integration tests
cargo test --all-targets

# Verify code formatting and linter
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 4. CLI Usage Guide

### 4.1 Initializing a Wallet

Initialize a new Native SegWit (BIP84) wallet with a generated mnemonic:
```bash
cargo run -- -d wallet.db -n regtest -t wpkh init
```

Initialize or recover a wallet using an existing 12-word mnemonic:
```bash
cargo run -- -d wallet.db -n regtest -t wpkh init --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

Initialize a Taproot (BIP86) wallet:
```bash
cargo run -- -d taproot_wallet.db -n regtest -t tr init
```

### 4.2 Deriving Addresses

Derive a new receive address (advances external keychain index `/0/*`):
```bash
cargo run -- -d wallet.db get-new-address
```

Derive a new change address (advances internal keychain index `/1/*`):
```bash
cargo run -- -d wallet.db get-change-address
```

List all derived addresses and their status:
```bash
cargo run -- -d wallet.db list-addresses
```

### 4.3 Inspecting Descriptors & Balance

Display wallet descriptors:
```bash
cargo run -- -d wallet.db show-descriptors
```

View wallet balance (confirmed and unconfirmed):
```bash
cargo run -- -d wallet.db get-balance
```

List all unspent transaction outputs in database:
```bash
cargo run -- -d wallet.db list-utxos
```

### 4.4 Syncing with Bitcoin Core

Sync wallet UTXOs with a running Bitcoin Core node via RPC:
```bash
cargo run -- -d wallet.db --rpc-url http://127.0.0.1:18443 --rpc-user user --rpc-pass password sync
```

### 4.5 Constructing & Sending Transactions

Build, sign, and display a raw transaction (without broadcasting):
```bash
cargo run -- -d wallet.db send \
  --recipient bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk \
  --amount-sats 25000 \
  --fee-rate 2 \
  --strategy largest-first
```

Build, sign, and broadcast directly to the network:
```bash
cargo run -- -d wallet.db send \
  --recipient bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk \
  --amount-sats 25000 \
  --fee-rate 2 \
  --broadcast
```

### 4.6 Running the Raw `rust-bitcoin` Script Demo

Run the low-level script construction and spending demonstration:
```bash
cargo run -- raw-demo
```

---

## 5. Architectural Deep Dive

### 5.1 BIP84 (P2WPKH) vs BIP86 (Taproot) Signing Mechanics

- **BIP84 (Native SegWit)**:
  - Output script: `0 <20-byte-key-hash>` (`OP_0 OP_PUSHBYTES_20 <hash160(compressed_pubkey)>`).
  - Sighash computation uses BIP143 digest with `SighashCache::p2wpkh_signature_hash`.
  - Signature is DER-encoded ECDSA with `0x01` (`SIGHASH_ALL`).
  - Witness stack: `[<ecdsa_signature>, <compressed_public_key>]`.

- **BIP86 (Taproot Key-Path)**:
  - Output script: `1 <32-byte-x-only-pubkey>` (`OP_1 OP_PUSHBYTES_32 <tweaked_x_only_pubkey>`).
  - Derives untweaked keypair, then tweaks it using `keypair.tap_tweak(&secp, None)` (empty script tree).
  - Sighash computation uses BIP341 digest with `SighashCache::taproot_key_spend_signature_hash` and `Prevouts::All`.
  - Signature is a 64-byte Schnorr signature (`TapSighashType::Default`).
  - Witness stack: `[<schnorr_signature>]` (smaller transaction weight).

### 5.2 SQLite Schema

```sql
CREATE TABLE wallet_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE keychain_indices (
    keychain INTEGER PRIMARY KEY, -- 0: external/receive, 1: internal/change
    next_index INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE derived_addresses (
    address TEXT PRIMARY KEY,
    script_pubkey_hex TEXT NOT NULL,
    is_change INTEGER NOT NULL,
    index_num INTEGER NOT NULL,
    used INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE utxos (
    txid TEXT NOT NULL,
    vout INTEGER NOT NULL,
    amount_sats INTEGER NOT NULL,
    script_pubkey_hex TEXT NOT NULL,
    address TEXT NOT NULL,
    is_change INTEGER NOT NULL,
    derivation_index INTEGER NOT NULL,
    height INTEGER,
    is_spent INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (txid, vout)
);

CREATE TABLE transactions (
    txid TEXT PRIMARY KEY,
    raw_tx_hex TEXT NOT NULL,
    fee_sats INTEGER,
    height INTEGER,
    is_outgoing INTEGER NOT NULL,
    timestamp INTEGER NOT NULL
);
```

### 5.3 Low-Level Script vs High-Level Descriptors

| Characteristic | Low-Level `rust-bitcoin` Script | Descriptor-Based Wallet |
| :--- | :--- | :--- |
| **Abstraction Level** | Raw opcodes (`OP_SHA256`, `OP_CHECKSIG`, etc.) | High-level pattern expressions (`wpkh(...)`, `tr(...)`) |
| **Address Generation** | Manual script hashing & address construction | Automated derivation paths (`/0/*`, `/1/*`) |
| **Signing Pipeline** | Explicit preimage, custom sighash digest, custom witness stack | Standardized sighash resolution & witness assembly |
| **Safety** | High risk of malleability or invalid witness order | Built-in verification and coin selection protection |

---

## 6. Limitations & Future Improvements

1. **Gap Limit Discovery**: Current sync scans derived addresses stored in the local database. Full BIP44 gap-limit scanning (lookahead of 20 unused addresses) can be integrated.
2. **Blockchain Reorganizations**: The database records confirmation heights. Adding block hash headers allows rollback handling during chain reorgs.
3. **SPV / P2P Sync**: Adding compact block filters (BIP157/158 Neutrino) would enable trustless mobile and light-client syncing without full RPC node requirements.
