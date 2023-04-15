use std::collections::HashMap;

use crate::monthgrid::MonthGrid;
use crate::types::{Account, amount::Amount};
use crate::journal::{Journal, JournalSummary};
use crate::iterators::transactionsbymonth::transactions_by_month;

// TODO: Assumes the same unit for all entries
pub fn balance_changes(journal: &Journal) -> MonthGrid<Account, Amount> {
    let summary = JournalSummary::from(journal);

    transactions_by_month(journal)
        .into_iter()
        .map(|(month, ts)| {
            let by_account = ts.iter()
                .flat_map(|transaction| &transaction.entries)
                .map(|entry| (entry.account.clone(), entry.amount.clone()))
                .fold(HashMap::<Account,Amount>::new(), |mut map, (account, amount)| {
                    map.entry(account)
                        .and_modify(|existing| existing.add(&amount))
                        .or_insert(amount);
                    map
                });
            (month, by_account)
        })
        .fold(MonthGrid::new(summary.first_month, summary.final_month), |mut grid, (month, by_account)| {
            for (account, amount) in by_account {
                grid.insert(account, month, amount);
            }
            grid
        })
}
