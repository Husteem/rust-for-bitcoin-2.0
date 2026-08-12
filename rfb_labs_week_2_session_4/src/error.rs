use std::fmt;

/// Every expected failure in the lending library.
///
/// This is the only file whose types are written for you. Nothing here should
/// ever be produced by a `panic!`, an `unwrap`, or an `expect` — these are
/// ordinary outcomes a caller is expected to handle.
#[derive(Debug, PartialEq, Eq)]
pub enum LibraryError {
    EmptyTitle,
    DuplicateItemId {
        id: u32,
    },
    DuplicateMemberId {
        id: u32,
    },
    ItemNotFound {
        id: u32,
    },
    MemberNotFound {
        id: u32,
    },
    ItemAlreadyOnLoan {
        id: u32,
        member_id: u32,
    },
    ItemNotOnLoan {
        id: u32,
    },
    ItemIsLost {
        id: u32,
    },
    BorrowLimitReached {
        member_id: u32,
        limit: usize,
    },
    InvalidReturnDay {
        day_borrowed: u32,
        day_returned: u32,
    },
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => write!(f, "Item title cannot be empty"),
            Self::DuplicateItemId { id } => {
                write!(f, "Library already contains an item with ID {}", id)
            }
            Self::DuplicateMemberId { id } => {
                write!(f, "Library already has a member registered with ID {}", id)
            }
            Self::ItemNotFound { id } => write!(f, "Item with ID {} not found", id),
            Self::MemberNotFound { id } => write!(f, "Member with ID {} not found", id),
            Self::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    f,
                    "Item {} is already checked out by member {}",
                    id, member_id
                )
            }
            Self::ItemNotOnLoan { id } => write!(f, "Item {} is not currently on loan", id),
            Self::ItemIsLost { id } => write!(f, "Item {} has been marked as lost", id),
            Self::BorrowLimitReached { member_id, limit } => {
                write!(
                    f,
                    "Member {} has reached their borrowing limit of {} items",
                    member_id, limit
                )
            }
            Self::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    f,
                    "Invalid return day: item was borrowed on day {}, but returned on day {}",
                    day_borrowed, day_returned
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}
