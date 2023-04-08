use std::collections::HashMap;

use crate::monthgrid::MonthGrid;
use crate::types::{Account, amount::Amount};
use crate::journal::{Journal, JournalSummary};
use crate::iterators::transactionsbymonth::transactions_by_month;

// TODO: Assumes the same unit for all entries
pub fn balance_changes(journal: &Journal) -> MonthGrid<Account, Amount> {
    let summary = JournalSummary::from(journal);
    let mut grid = MonthGrid::new(summary.first_month, summary.final_month);

    for (month, ts) in transactions_by_month(journal) {
        let mut by_account: HashMap<Account, Amount> = HashMap::new();

        for transaction in ts.iter() {
            for entry in &transaction.entries {
                by_account
                    .entry(entry.account.clone())
                    .and_modify(|amount| amount.add(&entry.amount))
                    .or_insert_with(|| entry.amount.clone());
            }
        }

        for (account, amount) in by_account {
            grid.insert(account, month, amount);
        }
    }

    grid
}
