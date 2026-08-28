# Lab 06 — Weight, virtual size, and fees

## Commands used

I executed the test suite for BIP141 weight units, virtual size calculation, and transaction fee comparison:

```bash
cargo test --test lab_06 -- --nocapture
```

## Terminal output

All 4 test assertions passed:

```text
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Observed calculation results (at 50 sat/vB):
- Legacy P2PKH Transaction (226 vB): `11,300 sats`
- Native SegWit P2WPKH Transaction (141 vB): `7,050 sats`
- Transaction Fee Savings: `4,250 sats` (~37.6% reduction)

## Evidence references

- Test suite implementation: `tests/lab_06.rs`
- Source logic: `src/labs/lab06_weight_fees.rs`
- Automated execution log: `grading/logs/lab_06.log`

## Explanation

SegWit (BIP141) replaced the legacy 1 MB block size limit with a 4,000,000 weight unit (WU) block weight limit, establishing precise accounting across transaction components:

1. Weight Formula:
   Weight = (Base Size * 3) + Total Serialized Size
   Equivalently:
   Weight = (Base Bytes * 4) + (Witness Bytes * 1)

2. Why SegWit is Not a Flat Whole-Transaction Discount:
   - Base transaction data (version, input TXID, vout index, sequence, output count, output amounts, and scriptPubKeys) must be stored by full nodes forever as part of the active UTXO set. Therefore, each base byte costs 4 weight units (1 virtual byte).
   - Witness data (signatures, public keys, and witness script stack items) only needs to be verified once during initial block validation and does not bloat the permanent UTXO database. Therefore, witness bytes receive a 75% discount, costing only 1 weight unit (0.25 virtual bytes).
   - Because the discount applies strictly to the witness section and not the base transaction structure, a transaction with a larger ratio of witness data (such as multisig or complex Taproot scripts) enjoys a proportionately greater fee reduction than a transaction dominated by numerous base outputs.
