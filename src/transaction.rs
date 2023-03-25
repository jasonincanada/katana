use lazy_static::lazy_static;
use chrono::NaiveDate;
use regex::Regex;
use std::fmt::{Display, Formatter, Result};

use crate::types::Account;
use crate::journal::types::{Amount, Line}; // TODO


/* Transaction */

// A transaction is a collection of 1 or more entries whose total dollar amount is zero

#[derive(Debug, Default, PartialEq)]
pub struct Transaction {
    pub date: NaiveDate,
    pub description: String,
    pub entries: Vec<Entry>
}

impl Transaction {

    pub fn total_cents(&self) -> i64 {
        self.entries.iter()
                    .map(|entry| entry.cents)
                    .sum()
    }

    // start a (temporarily empty) transaction with this date and description
    pub fn parse_date_and_description(line: &str) -> Option<Transaction> {
        let caps = DATE_REGEX.captures(line)?;
        let date = caps.name("date")?.as_str();
        let date = NaiveDate::parse_from_str(date, "%Y/%m/%d").ok()?;
        let description = caps.name("description")?.as_str().to_owned();

        Some(Transaction {
            date,
            description,
            entries: vec![],
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        writeln!(f, "{} {}", self.date, self.description)?;
        
        // transaction entries must be indented by at least one space
        for entry in &self.entries {
            writeln!(f, "    {}", entry)?;
        }

        Ok(())
    }
}

// compile the date/description regex once per run of the program
lazy_static! {
    static ref DATE_REGEX: Regex =
        Regex::new(r"^(?P<date>\d{4}/\d{2}/\d{2})\s+(?P<description>.+)$").unwrap();
}


/* Entry */

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub account: Account,
    pub cents  : i64
}

impl Entry {
    pub fn from_line(line: Line) -> Self {
        Entry {
            account: line.account,
            cents  : match line.amount {
                Amount::Cents(cents) => cents,
                                   _ => panic!("Expected an Amount::Cents for line.amount") // TODO
            }
        }
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}    ${:.2}", self.account, self.cents as f64 / 100.0)
    }
}


/* Tests */

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use super::{Entry, Transaction};

    #[test]
    fn test_parse_transaction_from_date_and_description() {
         let expected = 
            Some(Transaction {
                date: NaiveDate::from_ymd_opt(2023, 03, 11).unwrap(),
                description: "Meatball Sub".to_owned(),
                entries: vec![]
            });

        assert_eq!(Transaction::parse_date_and_description("2023/03/11 Meatball Sub"), expected);
        assert_eq!(Transaction::parse_date_and_description("2023/03/11    Meatball Sub"), expected);

        // transactions must be labeled
        assert_eq!(Transaction::parse_date_and_description("2023/03/11"), None);
    }

    fn create_entry(account: &str, cents: i64) -> Entry {
        Entry {
            account: account.to_string(),
            cents,
        }
    }

    #[test]
    fn test_fmt_display_positive_cents() {
        let entry = create_entry("account1", 1234);
        let formatted = format!("{}", entry);
        assert_eq!(formatted, "account1    $12.34");
    }

    #[test]
    fn test_fmt_display_negative_cents() {
        let entry = create_entry("account2", -5678);
        let formatted = format!("{}", entry);
        assert_eq!(formatted, "account2    $-56.78");
    }

    #[test]
    fn test_fmt_display_zero_cents() {
        let entry = create_entry("account3", 0);
        let formatted = format!("{}", entry);
        assert_eq!(formatted, "account3    $0.00");
    }

    #[test]
    fn test_fmt_display_trailing_zeros() {
        let entry = create_entry("account4", 4500);
        let formatted = format!("{}", entry);
        assert_eq!(formatted, "account4    $45.00");
    }

    #[test]
    fn test_fmt_display_no_cents() {
        let entry = create_entry("account5", 100);
        let formatted = format!("{}", entry);
        assert_eq!(formatted, "account5    $1.00");
    }
}
