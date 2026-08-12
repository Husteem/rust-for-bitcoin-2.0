# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits,
ownership, borrowing, collections, and `Result`-based error handling. No
Bitcoin and no external crates — just Rust.

The crate is intentionally incomplete. Search for `TODO` and implement each
part; do not change the public type names or function signatures.

## Recommended workflow

1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Complete Part 2 in `error.rs`, then Part 3 in `library.rs`.
3. Remove `#[ignore]` from the relevant test and run it.
4. Complete the traits in Part 4 and the two operations in Parts 5–6.
5. Run the ownership experiments and record the errors.
6. Build the demo in `main.rs`.
7. Add the remaining required tests yourself.

```bash
cargo test
cargo test -- --ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo test` checks the starter project. Ignored tests intentionally exercise
unfinished code; enable them progressively rather than leaving them ignored in
the submission.

## Written answers

### Part 7 Ownership Experiments

#### Experiment A Error

```text
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:15:20
   |
 8 |     let item = Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
14 |     library.add_item(item)?;
   |                      ---- value moved here
15 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move
```

**Explanation:** In Rust, passing a type that does not implement the `Copy` trait to a function by value transfers (moves) ownership of that variable. When we call `library.add_item(item)`, ownership of `item` moves into the library struct. Since the variable `item` is no longer valid in `main`'s scope, attempting to read `item.title` on the next line triggers a compile error.

#### Experiment B Error

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:17:5
   |
16 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
17 |     library.checkout(1, 100, 10)?;
   |     ^^^^^^^ mutable borrow occurs here
18 |     println!("{:?}", held);
   |                      ---- immutable borrow later used here
```

**Explanation:** Rust's borrow checker enforces that you cannot have a mutable borrow of a variable while an active immutable borrow exists. Calling `library.find_item(1)` borrows `library` immutably to return a reference `held` to an item inside it. This reference remains active until it is printed. Calling `library.checkout` requires a mutable borrow of `library`, causing a borrow conflict. This prevents data race issues like modifying the collection while holding references to its items.

---

### Questions

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?**
   Using a boolean like `is_on_loan` alongside options like `borrower_id: Option<u32>` allows for invalid states, such as having `is_on_loan` as false but still holding `borrower_id` as `Some(id)`. An enum makes these states mutually exclusive and guarantees that compile-time checks catch invalid states.

2. **What does `match` force you to do when a fourth `MediaKind` is added later?**
   Because match expressions in Rust must be exhaustive, adding a new media variant will trigger immediate compiler errors wherever `MediaKind` is matched, forcing us to explicitly write handling logic for the new variant.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title afterwards?**
   The constructed `Item` struct takes complete ownership of the title string once the constructor returns.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `add_item` needs to modify the library's internal catalog vector, which requires a mutable borrow of `self`. It takes `item` by value because the library takes ownership of the item to store it inside the catalog.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?**
   The passed `Item` is dropped and deallocated when the error occurs since it was moved into the function and its scope ended. While simple, this makes it impossible for the caller to recover the item and retry. An alternative would be returning the item back in the error variant (e.g., returning `Result<(), (Item, LibraryError)>`).

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   The library owns the items. Returning `Item` by value would attempt to move the item out of the catalog (which is forbidden) or require cloning it. Returning a reference `&Item` allows callers to inspect the item efficiently without transferring ownership.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**
   It specifies that the returned references to the items inside the vector borrow from the library, and their lifetime is bound to the library reference. They cannot outlive the library itself.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?**
   To enforce exclusive mutable access, Rust prevents mutably borrowing two separate parts of the library simultaneously. To structure this, we search for and store the indices of the item and member first, then access and modify them using index lookups inside a single scope.

9. **Why are `Library`'s fields private?**
   Keeping fields private enforces encapsulation. This guarantees that callers must use the public API methods, ensuring the loan status of items and member borrowed lists are modified together and never drift apart.

10. **What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?**
    It prevents repeating the overdue math (calculating late days and multiplying by the rate) across both `MediaKind` and `Item` trait implementations. Making it a free function would lose encapsulation and the clean method syntax (`item.late_fee_cents(days)`).

11. **Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.**
    Panics crash the program, which is bad for common user mistakes like checkout failures. Returning a `Result` allows graceful handling and logging. A panic is defensible in the return handler if we locate a loan status record pointing to a member ID that does not exist, as this indicates a broken internal library invariant.

12. **Which derive did you deliberately leave off a type, and why?**
    We left `Clone` and `Copy` off `Item` and `Member`. Books and members represent unique, real-world physical entities, and cloning them would lead to duplicate identities and tracking conflicts.

---

## Design notes

- **State Integrity**: To prevent state drift, we perform all checks up front before mutating any fields. The item's status update and the member's list update are done contiguously, guaranteeing that they are updated in sync.
- **Generic Search (Part 9)**: We implemented a generic `filter_items` helper taking `F: Fn(&Item) -> bool` and re-expressed `items_by_author` and `available_items` using it. This cleanly decoupled the filtering criteria from the collection traversal logic.

## Example output

```text
Alice checks out 'The Hobbit' on day 10...
Alice returns 'The Hobbit' on day 40...
Late return fee: 225 cents
Alice tries to checkout an unknown item (ID 999)...
Handled expected library error: Item with ID 999 not found
```
