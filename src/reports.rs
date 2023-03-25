use chrono::NaiveDate;
use std::fmt::{Display, Formatter, Result};
use crate::transaction::Entry;
use crate::types::Account;
use crate::journal::Journal;
use crate::transaction::Transaction;


pub struct ReportLine<'a> {
    date         : Option<NaiveDate>,          // only render the first date and
    description  : Option<&'a String>,         // description per transaction
    account      : &'a String,
    entry_cents  : i64,
    running_total: i64,
}

// a mask on a transaction that selects only certain entries, references to which are
// stored in a new vector. the selected entries may no longer balance to zero, though
// a reference to the underlying transaction is kept, which has all the original entries
struct FilteredTransaction<'a> {
    transaction: &'a Transaction,
    entries    : Vec<&'a Entry>
}

// run a report that shows each debit or credit to a given account, one debit/credit per
// line, and the running total on each line. only show the date/account once for a given
// date/account and leave blanks for the other entries
pub fn register_report<'a>(journal: &'a Journal,
                           account: &'a Account) -> Vec<ReportLine<'a>>
{
    let mut fts: Vec<FilteredTransaction> = journal.transactions
        .iter()
        .filter_map(|transaction| filter_entries_for(transaction, account))
        .collect();

    // TODO: we can skip this step if we know the journal is already sorted by date
    fts.sort_by_key(|ft| ft.transaction.date);
    
    let mut report_lines: Vec<ReportLine> = vec![];
    let mut running_total: i64 = 0;

    for filtered in fts {

        // for a multi-entry transaction we only want to print the date/description
        // for the first entry in the transaction, so track if it's the first
        let mut first = true;

        for entry in filtered.entries {
            running_total += entry.cents;

            let report_line = if first {
                first = false;
                ReportLine {
                    date       : Some( filtered.transaction.date),
                    description: Some(&filtered.transaction.description),
                    account    : &entry.account,
                    entry_cents: entry.cents,
                    running_total,
                }
            } else {
                ReportLine {
                    date       : None,
                    description: None,
                    account    : &entry.account,
                    entry_cents: entry.cents,
                    running_total,
                }
            };

            report_lines.push(report_line);
        }
    }

    report_lines
}

// find any entries in this transaction for this account, returning a list of
// references to them if there are any, or None if there's none
fn filter_entries_for<'a>(transaction: &'a Transaction,
                          account    : &'a Account) -> Option<FilteredTransaction<'a>>
{
    let entries: Vec<&Entry> =
        transaction.entries
                   .iter()
                   .filter(|entry| entry.account.as_str() == account)
                   .collect();

    if entries.is_empty() {
        return None
    }

    Some(FilteredTransaction {
        transaction,
        entries
    })
}

// 2023/03/18 Groceries                      assets:savings                      $-41.06       $399.64
// 2023/03/18 Crunchy Chicken Bowl           assets:savings                      $-16.10       $368.59

impl Display for ReportLine<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        let date = self.date
            .map(|date| format!("{}", date.format("%Y/%m/%d")))
            .unwrap_or_else(|| " ".repeat(10));

        let empty = "".to_string();
        let description = self.description.unwrap_or(&empty);

        let entry_amount  = format!("${:.2}", self.entry_cents as f64 / 100.0);
        let running_total = format!("${:.2}", self.running_total as f64 / 100.0);

        write!(
            f,
            "{} {:<30} {:<30} {:>10} {:>10}",
            date, description, self.account, entry_amount, running_total
        )
    }
}
