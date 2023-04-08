use lazy_static::lazy_static;
use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};

use crate::types::{Account, Units};
use crate::types::amount::{Amount, AmountType};


/* Transaction */

// a transaction is a collection of 2 or more entries whose total amount for each commodity is zero

#[derive(Debug, Default, PartialEq)]
pub struct Transaction {
    pub date: NaiveDate,
    pub description: String,
    pub entries: Vec<Entry>
}

impl Transaction {

    // get the total for each commodity (the different units) in this transaction
    pub fn totals(&self) -> HashMap<Units, Amount> {
        let mut map: HashMap<Units, Amount> = HashMap::new();

        for entry in &self.entries {
            if let Some(amount) = map.get_mut(&entry.amount.units) {
                amount.add(&entry.amount);
            } else {
                map.insert(entry.amount.units.clone(),
                           entry.amount.clone());
            }
        }
        map
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

lazy_static! {
    static ref DATE_REGEX: Regex =
        Regex::new(r"^(?P<date>\d{4}/\d{2}/\d{2})\s+(?P<description>.+)$").unwrap();
}


/* Entry */

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub account: Account,
    pub amount : Amount
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        // TODO: units
        match self.amount.amount {
            AmountType::Discrete(cents, _) => {
                write!(f, "{}    ${:.2}", self.account, cents as f64 / 100.0)
            }
            AmountType::Float(amt) => {
                write!(f, "{}    {:.3}", self.account, amt)
            }
        }
    }
}


/* Tests */

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use crate::types::amount::{Amount, AmountType};

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
            amount: Amount {
                amount: AmountType::Discrete(cents, 2),
                units: "$".to_owned()
            }
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
