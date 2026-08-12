use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Book { pages: u32 },
    Audiobook { minutes: u32 },
    Ebook { size_kb: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoanStatus {
    Available,
    OnLoan { member_id: u32, day_borrowed: u32 },
    Lost,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Item {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub kind: MediaKind,
    pub status: LoanStatus,
}

/// How long each kind of media may be kept, and what it costs to keep it too
/// long. Loan lengths are in whole days; fees are in whole cents.
pub trait LoanTerms {
    fn loan_days(&self) -> u32;

    fn daily_late_fee_cents(&self) -> u32;

    fn late_fee_cents(&self, days_held: u32) -> u32 {
        let allowed = self.loan_days();
        if days_held > allowed {
            (days_held - allowed) * self.daily_late_fee_cents()
        } else {
            0
        }
    }
}

impl Item {
    pub fn new(id: u32, title: String, author: String, kind: MediaKind) -> Self {
        Self {
            id,
            title,
            author,
            kind,
            status: LoanStatus::Available,
        }
    }
}

impl LoanTerms for MediaKind {
    fn loan_days(&self) -> u32 {
        match self {
            Self::Book { .. } => 21,
            Self::Audiobook { .. } => 14,
            Self::Ebook { .. } => 7,
        }
    }

    fn daily_late_fee_cents(&self) -> u32 {
        match self {
            Self::Ebook { .. } => 0,
            _ => 25,
        }
    }
}

impl LoanTerms for Item {
    fn loan_days(&self) -> u32 {
        self.kind.loan_days()
    }

    fn daily_late_fee_cents(&self) -> u32 {
        self.kind.daily_late_fee_cents()
    }
}

impl fmt::Display for MediaKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Book { pages } => write!(f, "Book ({} pages)", pages),
            Self::Audiobook { minutes } => write!(f, "Audiobook ({} minutes)", minutes),
            Self::Ebook { size_kb } => write!(f, "Ebook ({} KB)", size_kb),
        }
    }
}

impl fmt::Display for LoanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::OnLoan {
                member_id,
                day_borrowed,
            } => {
                write!(
                    f,
                    "On Loan (to member {} since day {})",
                    member_id, day_borrowed
                )
            }
            Self::Lost => write!(f, "Lost"),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[ID {}] '{}' by {} ({}) - {}",
            self.id, self.title, self.author, self.kind, self.status
        )
    }
}
