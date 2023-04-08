use chrono::NaiveDate;
use std::collections::HashMap;
use std::fmt;
use crate::transaction::Entry;
use crate::types::{Account, amount::Amount, Units};
use crate::journal::Journal;
use crate::transaction::Transaction;


// one line of the register report
pub struct ReportLine<'a> {
    date         : Option<NaiveDate>,          // only render the first date and
    description  : Option<&'a String>,         // description per transaction
    account      : &'a String,
    amount       : String,
    running_total: String,
}

// a mask on a transaction that selects only certain entries, references to which are
// stored in a new vector. the selected entries may no longer balance to zero, though
// a reference to the underlying transaction is kept, which has all the original entries
struct FilteredTransaction<'a> {
    transaction: &'a Transaction,
    entries    : Vec<&'a Entry>
}

// Generates a register report for a given account, showing each debit or credit
// transaction with a running total for each line. Displays the date and description
// information only once for each transaction, leaving blanks for the other lines.
pub fn register_report<'a>(journal: &'a Journal,
                           account: &'a Account) -> Vec<ReportLine<'a>>
{
    let fts = filter_by_account(&journal.transactions, account);
    let mut report_lines: Vec<ReportLine> = vec![];
    let mut running_totals: HashMap<Units, Amount> = HashMap::new();

    for filtered in fts {

        // for a multi-entry transaction we only want to print the date/description
        // for the first line
        let mut is_first_entry = true;

        for entry in filtered.entries {
            update_running_totals(&mut running_totals, entry);

            let units = &entry.amount.units;
            let running_total = running_totals.get(units).unwrap().clone();
            let report_line = create_report_line(filtered.transaction,
                                                 entry,
                                                 running_total,
                                                 is_first_entry);
            
            report_lines.push(report_line);
            is_first_entry = false;
        }
    }

    report_lines
}

fn create_report_line<'a>(transaction   : &'a Transaction,
                          entry         : &'a Entry,
                          running_total : Amount,
                          is_first_entry: bool) -> ReportLine<'a>
{
    ReportLine {
        date         : if is_first_entry { Some(transaction.date) } else { None },
        description  : if is_first_entry { Some(&transaction.description) } else { None },
        account      : &entry.account,
        amount       : entry.amount.to_string(),
        running_total: running_total.to_string()
    }
}

// Filters the transactions by the given account and returns a vector of FilteredTransaction.
// For each transaction, it checks if there are any entries associated with the account.
// If there are any, it creates a FilteredTransaction with a reference to the transaction
// and the relevant entries. If not, it skips the transaction.
fn filter_by_account<'a>(transactions: &'a [Transaction],
                         account     : &'a Account) -> Vec<FilteredTransaction<'a>>
{
    transactions
        .iter()
        .filter_map(|transaction| {
            let entries: Vec<&Entry> =
                transaction.entries
                           .iter()
                           .filter(|entry| entry.account.as_str() == account)
                           .collect();

            if entries.is_empty() {
                return None;
            }

            Some(FilteredTransaction {
                transaction,
                entries
            })
        })
        .collect()
}

fn update_running_totals(totals: &mut HashMap<Units, Amount>,
                         entry : &Entry)
{
    let units = &entry.amount.units;

    if let Some(amount) = totals.get_mut(units) {
        amount.add(&entry.amount);
    } else {
        totals.insert(units.clone(), entry.amount.clone());
    }
}


// 2023/03/18 Groceries                      assets:savings                      $-41.06       $399.64
// 2023/03/18 Crunchy Chicken Bowl           assets:savings                      $-16.10       $368.59

impl fmt::Display for ReportLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let date = self.date
            .map(|date| format!("{}", date.format("%Y/%m/%d")))
            .unwrap_or_else(|| " ".repeat(10));

        let empty = "".to_string();
        let description = self.description.unwrap_or(&empty);

        write!(
            f,
            "{} {:<30} {:<30} {:>10} {:>10}",
            date, description, self.account, self.amount, self.running_total
        )
    }
}

