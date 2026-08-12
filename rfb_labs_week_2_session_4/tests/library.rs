use rfb_labs_week_2_session_4::{Item, Library, LibraryError, LoanStatus, MediaKind, Member};

fn library_with_items() -> Library {
    let mut library = Library::new();

    for (id, title, author, kind) in [
        (1, "Dune", "Frank Herbert", MediaKind::Book { pages: 320 }),
        (
            2,
            "Children of Dune",
            "Frank Herbert",
            MediaKind::Book { pages: 180 },
        ),
        (
            3,
            "Project Hail Mary",
            "Andy Weir",
            MediaKind::Audiobook { minutes: 540 },
        ),
        (
            4,
            "The Rust Programming Language",
            "Steve Klabnik",
            MediaKind::Ebook { size_kb: 1_200 },
        ),
    ] {
        library
            .add_item(Item::new(id, title.into(), author.into(), kind))
            .unwrap();
    }

    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    library
}

// These tests are ignored so the starter repository builds before students
// implement the TODOs. Remove `#[ignore]` from one test at a time while working.

#[test]
fn checkout_updates_both_the_item_and_the_member() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.find_item(1).unwrap().status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 5,
        }
    );
    assert_eq!(library.find_member(100).unwrap().borrowed_item_ids, vec![1]);
}

#[test]
fn a_member_cannot_exceed_the_borrow_limit() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();
    library.checkout(3, 100, 0).unwrap();

    assert_eq!(
        library.checkout(4, 100, 0),
        Err(LibraryError::BorrowLimitReached {
            member_id: 100,
            limit: 3,
        })
    );
}

#[test]
fn returning_a_book_late_charges_a_daily_fee() {
    let mut library = library_with_items();

    // A book may be kept 21 days. Held for 30, so 9 days are overdue.
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 40), Ok(9 * 25));
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
    assert!(
        library
            .find_member(100)
            .unwrap()
            .borrowed_item_ids
            .is_empty()
    );
}

#[test]
fn searching_by_author_borrows_rather_than_clones() {
    let library = library_with_items();

    let found = library.items_by_author("Frank Herbert");

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].title, "Dune");
    // `found` holds references into `library`, so these are the same item.
    assert!(std::ptr::eq(found[0], library.find_item(1).unwrap()));
}

#[test]
fn cannot_checkout_twice() {
    let mut library = library_with_items();
    library.checkout(1, 100, 0).unwrap();

    assert_eq!(
        library.checkout(1, 100, 0),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100
        })
    );
}

#[test]
fn on_time_return_owes_nothing() {
    let mut library = library_with_items();
    library.checkout(1, 100, 10).unwrap();
    // Book has 21 days checkout period. 10 + 21 = 31.
    assert_eq!(library.return_item(1, 31), Ok(0));
}

#[test]
fn ebook_returned_late_owes_nothing() {
    let mut library = library_with_items();
    // 4 is the ebook "The Rust Programming Language"
    library.checkout(4, 100, 10).unwrap();
    // Ebook has 7 days. Returned at 100 (90 days late). Still owes nothing.
    assert_eq!(library.return_item(4, 100), Ok(0));
}

#[test]
fn author_search_returns_borrowed_items() {
    let mut library = library_with_items();
    library.checkout(1, 100, 0).unwrap();

    let found = library.items_by_author("Frank Herbert");
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|i| i.title == "Dune"));
}

#[test]
fn test_checkout_validation_errors() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(999, 100, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );

    assert_eq!(
        library.checkout(1, 999, 0),
        Err(LibraryError::MemberNotFound { id: 999 })
    );
}

#[test]
fn test_lost_item_checkout() {
    let mut library = Library::new();
    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();
    let mut item = Item::new(
        5,
        "Lost Book".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;
    library.add_item(item).unwrap();

    assert_eq!(
        library.checkout(5, 100, 0),
        Err(LibraryError::ItemIsLost { id: 5 })
    );
}

#[test]
fn test_duplicate_add_and_empty_title() {
    let mut library = Library::new();

    let item_empty = Item::new(
        1,
        "".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    assert_eq!(library.add_item(item_empty), Err(LibraryError::EmptyTitle));

    let item1 = Item::new(
        1,
        "Book 1".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    library.add_item(item1).unwrap();
    let item2 = Item::new(
        1,
        "Book 2".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    assert_eq!(
        library.add_item(item2),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );

    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();
    assert_eq!(
        library.register_member(Member::new(100, "Bob".into())),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    );
}

#[test]
fn test_return_validation_errors() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(999, 10),
        Err(LibraryError::ItemNotFound { id: 999 })
    );

    let mut item_lost = Item::new(
        5,
        "Lost".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    item_lost.status = LoanStatus::Lost;
    library.add_item(item_lost).unwrap();
    assert_eq!(
        library.return_item(5, 10),
        Err(LibraryError::ItemIsLost { id: 5 })
    );

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );

    library.checkout(1, 100, 10).unwrap();
    assert_eq!(
        library.return_item(1, 5),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 10,
            day_returned: 5
        })
    );
}

#[test]
fn test_longest_loan_item() {
    let library = library_with_items();
    let longest = library.longest_loan_item().unwrap();
    use rfb_labs_week_2_session_4::LoanTerms;
    assert_eq!(longest.loan_days(), 21);
}
