use crate::catalogue::Item;
use crate::error::LibraryError;
use crate::member::Member;

pub const MAX_ITEMS_PER_MEMBER: usize = 3;

/// Owns every item and every member.
///
/// The fields are private because the library is responsible for keeping an
/// item's `LoanStatus` and a member's borrowed-id list in agreement. Callers
/// reach the data through the borrowing lookups below.
// TODO(Part 3): delete this attribute once your lookups actually read the
// fields. It is here only so the untouched starter crate compiles clean.
#[derive(Debug, Default)]
pub struct Library {
    items: Vec<Item>,
    members: Vec<Member>,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_item(&mut self, item: Item) -> Result<(), LibraryError> {
        if item.title.trim().is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.items.iter().any(|i| i.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        if self.members.iter().any(|m| m.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    pub fn filter_items<F>(&self, filter: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|i| filter(i)).collect()
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.filter_items(|i| i.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.filter_items(|i| matches!(i.status, crate::catalogue::LoanStatus::Available))
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        use crate::catalogue::LoanTerms;
        self.items.iter().max_by_key(|i| i.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        let item_idx = self
            .items
            .iter()
            .position(|i| i.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let member_idx = self
            .members
            .iter()
            .position(|m| m.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        if matches!(
            self.items[item_idx].status,
            crate::catalogue::LoanStatus::Lost
        ) {
            return Err(LibraryError::ItemIsLost { id: item_id });
        }

        if let crate::catalogue::LoanStatus::OnLoan {
            member_id: borrower_id,
            ..
        } = self.items[item_idx].status
        {
            return Err(LibraryError::ItemAlreadyOnLoan {
                id: item_id,
                member_id: borrower_id,
            });
        }

        if self.members[member_idx].borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        self.items[item_idx].status = crate::catalogue::LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };
        self.members[member_idx].borrowed_item_ids.push(item_id);

        Ok(())
    }

    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        use crate::catalogue::LoanTerms;

        let item_idx = self
            .items
            .iter()
            .position(|i| i.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        if matches!(
            self.items[item_idx].status,
            crate::catalogue::LoanStatus::Lost
        ) {
            return Err(LibraryError::ItemIsLost { id: item_id });
        }

        let (borrower_id, day_borrowed) = match self.items[item_idx].status {
            crate::catalogue::LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => (member_id, day_borrowed),
            _ => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
        };

        let days_held = day
            .checked_sub(day_borrowed)
            .ok_or(LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned: day,
            })?;

        let fee = self.items[item_idx].late_fee_cents(days_held);

        self.items[item_idx].status = crate::catalogue::LoanStatus::Available;

        if let Some(m) = self.members.iter_mut().find(|m| m.id == borrower_id) {
            m.borrowed_item_ids.retain(|&id| id != item_id);
        }

        Ok(fee)
    }
}
