# Rust for Bitcoin 2.0 — Week 2

Build a simplified Bitcoin transaction model while practising structs, enums,
traits, ownership, borrowing, collections, and `Result`-based error handling.

The crate is intentionally incomplete. Search for `TODO` and implement each part;
do not change the public type names or function signatures.

## Recommended workflow

1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Complete Parts 3–5 in `transaction.rs` and `error.rs`.
3. Remove `#[ignore]` from the relevant test and run it.
4. Complete the traits and borrowing functions in Parts 6–7.
5. Build the payment example in `main.rs`.
6. Complete UTXO selection and its tests.
7. Add the remaining required tests yourself.

```bash
cargo test
cargo test -- --ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo test` checks the starter project. Ignored tests intentionally exercise
unfinished code; enable them progressively rather than leaving them ignored in the
submission.

## Written answers

### Part 7 Ownership Compiler Error

```text
error[E0515]: cannot return reference to local variable `output`
  --> src/transaction.rs:175:5
   |
175|     &output
   |     ^^^^^^^ returns a reference to data owned by the current function
```

**Explanation:** In Rust, a function cannot return a reference to a value created locally inside its own scope. When the function finishes execution, local variables are dropped, which would leave a dangling pointer. By annotating the function with explicit lifetime `'a` (`fn find_outputs_for_recipient<'a>(transaction: &'a Transaction, recipient: &str) -> Vec<&'a TxOutput>`), we return references borrowing directly from the caller's `Transaction` struct without creating local owned copies.

---

### Questions

1. **What is a Bitcoin transaction input?**
   A reference to a previously unspent output (an Outpoint containing `txid:vout`) along with unlocking script or witness data proving authorization to spend those coins.

2. **What is a Bitcoin transaction output?**
   A locking script (`scriptPubKey`) defining spending requirements, paired with an integer amount of satoshis being transferred.

3. **What is a UTXO?**
   An Unspent Transaction Output. It represents an indivisible coin created by a previous transaction output that has not yet been consumed as an input in a subsequent valid block.

4. **What does an outpoint identify?**
   A unique 2-tuple coordinate (`txid:vout`) pointing to a specific transaction output in global blockchain history.

5. **How is a transaction fee calculated?**
   Fee is calculated as total input satoshis minus total output satoshis (fee = total inputs - total outputs). It is implicit (unassigned leftover value) claimed by the miner in their coinbase block reward.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point types (like `f64`) introduce IEEE-754 precision rounding errors (such as `0.1 + 0.2 = 0.30000000000000004`), which can cause consensus splits or rounding bugs across nodes. Integer satoshis (`1 BTC = 100,000,000 sats`) ensure exact, deterministic arithmetic.

7. **Why does `total_input_value()` borrow `self`?**
   Summing input values only requires read-only inspection of the input fields. Taking an immutable reference (`&self`) allows the caller to retain ownership of the `Transaction` struct after querying its total value.

8. **Why does `add_input()` take `&mut self`?**
   `add_input()` mutates the state of the `Transaction` struct by pushing a new element into its `inputs` vector. In Rust, mutating internal struct fields requires a mutable borrow `&mut self`.

9. **What happens when an input is moved into a transaction?**
   Ownership of the `InputKind` instance transfers from the caller into the `Transaction` struct's `inputs` vector. The caller can no longer access or modify the original input variable unless borrowing it back from the transaction.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    `panic!` aborts the current execution thread, which can crash node daemon P2P message-processing loops when parsing untrusted data from peers. Returning `Result<T, TransactionError>` allows callers to handle invalid data gracefully, log errors, or ban malformed peers without crashing the process.

11. **How do enums help model regular and coinbase inputs?**
    `InputKind` uses Rust enum variants (`Regular` containing `previous_output` and `sequence`, `Coinbase` containing `block_height` and `reward`). Pattern matching forces the compiler to ensure both regular and coinbase input structures are explicitly handled across all code paths.

12. **How does the `BitcoinValue` trait reduce duplication?**
    The `BitcoinValue` trait abstracts `.value()` access and provides a default method `.value_in_btc()`. Implementing `BitcoinValue` for `TxOutput` and `InputKind` avoids duplicating BTC conversion math across multiple types.

## Design notes

- **UTXO Selection Trade-Offs:** The `select_utxos` implementation uses a simple first-fit algorithm over a borrowed slice. It accumulates available UTXOs in slice order until reaching the required target amount. While simple and $O(N)$ fast, it does not attempt to minimize change output size or optimize privacy (which coin selection algorithms like Branch and Bound or Knapsack do).
- **Borrowing Efficiency:** `select_utxos`, `highest_value_output`, and `find_outputs_for_recipient` return borrowed references (`&Utxo`, `&TxOutput`), avoiding unnecessary memory allocations or `.clone()` calls.

## Example output

```text
Transaction is valid!
Transaction (v2, locktime: 0):
  Inputs (2): 120000
  Outputs (2): 118000
  Total Input: 120000 sats (0.00120000 BTC)
  Total Output: 118000 sats (0.00118000 BTC)
  Fee: 2000 sats (0.00002000 BTC)
```
