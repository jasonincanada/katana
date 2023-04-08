use crate::journal::{Journal, JournalSummary};
use crate::transaction::Transaction;
use crate::types::monthyear::MonthYear;

/* Journal -> (Month, Month) -> Iterator< (MonthYear,Item=&[Transaction]) > */

/// Iterate over a journal starting from a certain month/year, returning slices
/// of transactions that all fall within the same month. Assumes the journal
/// is sorted by transaction date because it uses a binary search to locate dates
pub struct TransactionsByMonth<'a> {
    journal: &'a Journal,
    current_month: MonthYear,
    final_month  : MonthYear
}

impl<'a> Iterator for TransactionsByMonth<'a> {
    type Item = (MonthYear, &'a [Transaction]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_month > self.final_month {
            return None;
        }

        let transactions = &self.journal.transactions;
        let start = transactions.partition_point(|t| {
            MonthYear::from_naivedate(t.date) < self.current_month
        });

        let next_month = self.current_month.next_month();
        let end = transactions.partition_point(|t| {
            MonthYear::from_naivedate(t.date) < next_month
        });

        let month = self.current_month;
        self.current_month = next_month;
        Some((month, &transactions[start..end]))
    }
}

pub fn transactions_by_month(journal: &Journal) -> TransactionsByMonth {
    let summary = JournalSummary::from(journal);

    TransactionsByMonth {
        journal,
        current_month: summary.first_month,
        final_month  : summary.final_month
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use crate::{journal::Journal, transaction::Transaction, types::monthyear::MonthYear};
    use super::TransactionsByMonth;

    fn sample_journal() -> Journal {
        Journal {
            transactions: vec![
                Transaction { date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(), entries: vec![], description: "".to_owned() },
                Transaction { date: NaiveDate::from_ymd_opt(2022, 1, 15).unwrap(), entries: vec![], description: "".to_owned() },
                Transaction { date: NaiveDate::from_ymd_opt(2022, 2, 5).unwrap(), entries: vec![], description: "".to_owned() },
                Transaction { date: NaiveDate::from_ymd_opt(2022, 2, 25).unwrap(), entries: vec![], description: "".to_owned() },
                Transaction { date: NaiveDate::from_ymd_opt(2022, 3, 10).unwrap(), entries: vec![], description: "".to_owned() },
                Transaction { date: NaiveDate::from_ymd_opt(2022, 3, 20).unwrap(), entries: vec![], description: "".to_owned() },
            ],
        }
    }

    #[test]
    fn test_iterator() {
        let journal = sample_journal();

        let iterator = TransactionsByMonth {
            journal: &journal,
            current_month: MonthYear { month: 1, year: 2022 },
            final_month: MonthYear { month: 3, year: 2022 }
        };

        let month_slices: Vec<(MonthYear, &[Transaction])> = iterator.collect();

        assert_eq!(month_slices.len(), 3);

        assert_eq!(month_slices[0].1, &journal.transactions[0..2]);
        assert_eq!(month_slices[1].1, &journal.transactions[2..4]);
        assert_eq!(month_slices[2].1, &journal.transactions[4..6]);
    }

    #[test]
    fn test_iterator_no_transactions() {
        let journal = sample_journal();

        let iterator = TransactionsByMonth {
            journal: &journal,
            current_month: MonthYear { month: 4, year: 2022 },
            final_month: MonthYear { month: 6, year: 2022 }
        };

        let month_slices: Vec<(MonthYear, &[Transaction])> = iterator.collect();

        assert_eq!(month_slices.len(), 3);
        assert_eq!(month_slices[0].1, &[]);
        assert_eq!(month_slices[1].1, &[]);
        assert_eq!(month_slices[2].1, &[]);
    }

    fn sample_journal_empty_slice_middle() -> Journal {
        Journal {
            transactions: vec![
                Transaction { date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(), entries: vec![], description: "".to_owned() },
                Transaction { date: NaiveDate::from_ymd_opt(2022, 1, 15).unwrap(), entries: vec![], description: "".to_owned() },
                // skip february
                Transaction { date: NaiveDate::from_ymd_opt(2022, 3, 10).unwrap(), entries: vec![], description: "".to_owned() },
                Transaction { date: NaiveDate::from_ymd_opt(2022, 3, 20).unwrap(), entries: vec![], description: "".to_owned() },
            ],
        }
    }

    #[test]
    fn test_iterator_empty_slice_middle() {
        let journal = sample_journal_empty_slice_middle();

        let iterator = TransactionsByMonth {
            journal: &journal,
            current_month: MonthYear { month: 1, year: 2022 },
            final_month: MonthYear { month: 3, year: 2022 }
        };

        let month_slices: Vec<(MonthYear, &[Transaction])> = iterator.collect();

        assert_eq!(month_slices.len(), 3);

        assert_eq!(month_slices[0].1, &journal.transactions[0..2]);
        assert_eq!(month_slices[1].1, &[]); // Expect an empty slice for February
        assert_eq!(month_slices[2].1, &journal.transactions[2..4]);
    }
}
