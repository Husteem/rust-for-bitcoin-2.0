//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "The Hobbit".into(),
        "J.R.R. Tolkien".into(),
        MediaKind::Book { pages: 310 },
    ))?;
    library.add_item(Item::new(
        2,
        "1984".into(),
        "George Orwell".into(),
        MediaKind::Ebook { size_kb: 800 },
    ))?;

    library.register_member(Member::new(100, "Alice".into()))?;

    println!("Alice checks out 'The Hobbit' on day 10...");
    library.checkout(1, 100, 10)?;

    println!("Alice returns 'The Hobbit' on day 40...");
    let fee = library.return_item(1, 40)?;
    println!("Late return fee: {} cents", fee);

    println!("Alice tries to checkout an unknown item (ID 999)...");
    match library.checkout(999, 100, 40) {
        Ok(_) => println!("Successfully checked out unknown item!"),
        Err(e) => {
            println!("Handled expected library error: {}", e);
        }
    }

    Ok(())
}
