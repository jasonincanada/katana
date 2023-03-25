pub mod types;

use std::str::FromStr;
use std::fmt::{Display, Formatter, Result};
use crate::transaction::{Transaction, Entry};
use crate::common::is_all_whitespace;
use types::Line;


/* Journal */

#[derive(Debug, PartialEq)]
pub struct Journal {
    pub transactions: Vec<Transaction>
}

#[derive(Debug, PartialEq)]
pub enum ParseJournalError {
    EntryLineMustStartWithSpace,
}

impl Display for ParseJournalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            ParseJournalError::EntryLineMustStartWithSpace =>
                write!(f, "First character of a debit/credit line must be a space or tab"),
        }
    }
}

impl Journal {

    //
    // 2023/03/15 Sandwich
    //    assets:savings                     $-6.76
    //    assets:cash                           $-1
    //    expenses:tips                          $1
    //    expenses:food:tim-hortons
    //
    pub fn from_lines(lines: std::str::Lines) -> std::result::Result<Journal, ParseJournalError> {

        let mut journal    : Vec<Transaction>    = vec![];
        let mut transaction: Option<Transaction> = None;
        let mut blank      : Option<Line>        = None; // we can have up to one unspecified
                                                         // amount per transaction
        for line in lines {
            let (line, _) = split_off_comment(line);

            // "2023/03/15 Sandwich"
            if let Some(trans) = Transaction::parse_date_and_description(&line) {
                
                // this line is the header for a new transaction, so check if we
                // have one already. process it and move it into the journal if so
                finalize_transaction(&mut transaction,
                                     &mut blank,
                                     &mut journal);

                // the new transaction is the one we're now adding entries to
                transaction = Some(trans);
                continue
            }

            //
            if is_all_whitespace(&line) {
                continue
            }

            if !line.chars().next().unwrap().is_whitespace() {
                return Err(ParseJournalError::EntryLineMustStartWithSpace)
            }

            //    assets:savings    $-6.76
            if let Ok(line) = Line::from_str(&line) {
                process_line(line,
                             &mut transaction,
                             &mut blank);
                continue
            }

            panic!("Couldn't process this line: '{}'", line)
        }

        // Add the last pending transaction to the journal, if there is one
        finalize_transaction(&mut transaction,
                             &mut blank,
                             &mut journal);

        Ok(Journal { transactions: journal })
    }
}

fn process_line(line       : Line,
                transaction: &mut Option<Transaction>,
                blank      : &mut Option<Line>)
{
    if transaction.is_none() {
        panic!("Can't have a debit/credit outside a transaction")
    }

    if line.amount.is_blank()
    {
        if blank.is_some() {
            panic!("Two blank amounts in one transaction");
        }
        // update the variable behind the reference, it now owns this line
        *blank = Some(line);
    }
    else
    {
        let entry = Entry::from_line(line);
        // borrow a mutable reference to the underlying transaction,
        // with it add this entry to the transaction's list of entries
        transaction.as_mut().unwrap().entries.push(entry);
    }
}

// if we have a transaction on hand, balance it and move it to the journal
fn finalize_transaction(transaction: &mut Option<Transaction>,
                        blank      : &mut Option<Line>,
                        journal    : &mut Vec<Transaction>)
{
    if let Some(mut t) = transaction.take() {
        balance_transaction(blank, &mut t);
        journal.push(t);
    }
}

// balance this transaction if necessary by checking if there's an account line with no
// amount. if so, set the amount to balance out the other entries in the transaction
fn balance_transaction(blank      : &mut Option<Line>,
                       transaction: &mut Transaction)
{
    // take() is used here to take ownership of blank's value since it's an Account
    // and we need to use its String (account name) to build the Entry struct.
    // blank is set back to None for us by the call to take()
    if let Some(line) = blank.take() {
        let total = transaction.total_cents();

        // create a new entry with the amount that balances the overall transaction to zero
        transaction.entries.push(Entry {
            account: line.account,
            cents  : -total
        });
    }
    else
    {
        // we don't have a blank line, so check to make sure the entries balance
        if transaction.total_cents() != 0 {
            panic!("Unbalanced transaction: {}", transaction);
        }
    }
}

// split off any comment from the end of a journal line and return both parts.
// any comment starts at the first ; and continues for the rest of the line
fn split_off_comment(line: &str) -> (String, Option<String>) {

    match line.splitn(2, ';')
              .collect::<Vec<&str>>()
              .as_slice()
    {
        [line, comment] => (line.to_string(), Some(comment.to_string())),
        [line         ] => (line.to_string(), None),
                      _ => unreachable!()
    }
}


/* Tests */

#[cfg(test)]
mod tests {
    use super::{Entry, Line, Journal, Transaction,
                finalize_transaction, process_line, split_off_comment};
    use crate::journal::ParseJournalError;
    use crate::journal::types::Amount; // TODO

    // Journal::from_lines()

    #[test]
    fn test_journal_from_lines() {
        let journal = 
r#"
2023/03/17 Ham Sub
    assets:savings $-12.46
    expenses:tips $1.62
    expenses:food:subway $10.84

2023/03/17 HelloFresh
    expenses:food:hello-fresh           $82.99
    credit:visa
"#;
        let journal = Journal::from_lines(journal.lines()).unwrap();
        assert_eq!(journal.transactions.len(), 2);
    }

    #[test]
    fn test_journal_from_lines_backwards() {
        let journal = 
r#"
2023/03/17 HelloFresh
    expenses:food:hello-fresh           $82.99
    credit:visa

2023/03/17 Ham Sub
    assets:savings $-12.46
    expenses:tips $1.62
    expenses:food:subway $10.84
"#;
        let journal = Journal::from_lines(journal.lines()).unwrap();
        assert_eq!(journal.transactions.len(), 2);
    }

    #[test]
    fn test_journal_from_lines_no_leading_whitespace() {
        let journal = 
r#"
2023/03/17 HelloFresh
expenses:food:hello-fresh           $82.99
    credit:visa                         $-82.98
"#;
        assert_eq!(Journal::from_lines(journal.lines()),
                   Err(ParseJournalError::EntryLineMustStartWithSpace));
    }

    #[test]
    #[should_panic]
    fn test_journal_from_lines_unbalanced() {
        let journal = 
r#"
2023/03/17 HelloFresh
    expenses:food:hello-fresh           $82.99
    credit:visa                         $-82.98
"#;
        Journal::from_lines(journal.lines()).ok();
    }

    #[test]
    #[should_panic]
    fn test_journal_from_lines_two_blanks() {
        let journal = 
r#"
2023/03/17 HelloFresh
    expenses:food:hello-fresh
    credit:visa
"#;
        Journal::from_lines(journal.lines()).ok();
    }

    #[test]
    #[should_panic]
    fn test_journal_from_lines_amount_outside_transaction() {
        let journal = 
r#"
    expenses:food:hello-fresh  $89.99

2023/03/17 HelloFresh
    expenses:food:hello-fresh  $89.99
    credit:visa
"#;
        Journal::from_lines(journal.lines()).ok();
    }

    #[test]
    fn test_split_off_comment() {
        assert_eq!(split_off_comment("  ;comment"), ("  ".to_string(), Some("comment".to_string())));
        assert_eq!(split_off_comment(";comment"),   ("".to_string(), Some("comment".to_string())));
        assert_eq!(split_off_comment("test;"),      ("test".to_string(), Some("".to_string())));
        assert_eq!(split_off_comment("test; "),     ("test".to_string(), Some(" ".to_string())));
        assert_eq!(split_off_comment(";"),          ("".to_string(), Some("".to_string())));
        assert_eq!(split_off_comment("no comment"), ("no comment".to_string(), None));
        assert_eq!(split_off_comment(" "),          (" ".to_string(), None));
        assert_eq!(split_off_comment(""),           ("".to_string(), None));
    }


    // process_line()

    #[test]
    #[should_panic(expected = "Can't have a debit/credit outside a transaction")]
    fn test_process_line_panic_no_transaction() {
        let line = Line {
            account: "TestAccount".to_string(),
            amount: Amount::Blank,
        };
        let mut transaction: Option<Transaction> = None;
        let mut blank: Option<Line> = None;

        process_line(line, &mut transaction, &mut blank)
    }

    #[test]
    #[should_panic(expected = "Two blank amounts in one transaction")]
    fn test_process_line_panic_two_blank_amounts() {
        let line = Line {
            account: "TestAccount".to_string(),
            amount: Amount::Blank,
        };
        let mut transaction = Some(Transaction::default());
        // clone the blank transaction line so we have two blank transactions
        let mut blank = Some(line.clone());

        process_line(line, &mut transaction, &mut blank)
    }

    #[test]
    fn test_process_line_blank_amount() {
        let line = Line {
            account: "TestAccount".to_string(),
            amount: Amount::Blank,
        };
        let mut transaction = Some(Transaction::default());
        let mut blank: Option<Line> = None;

        process_line(line.clone(), &mut transaction, &mut blank);
        assert_eq!(blank.unwrap().account, line.account);
    }

    #[test]
    fn test_process_line_regular_amount() {
        let line = Line {
            account: "TestAccount".to_string(),
            amount: Amount::Cents(125),
        };
        let mut transaction = Some(Transaction::default());
        let mut blank: Option<Line> = None;

        process_line(line.clone(), &mut transaction, &mut blank);

        let entry = transaction.unwrap().entries.pop().unwrap();
        assert_eq!(entry.account, line.account);
    }


    // move_transaction()

    #[test]
    fn test_move_transaction_blank_line() {
        let line = Line {
            account: "TestAccount".to_string(),
            amount: Amount::Blank,
        };
        let mut transaction = Some(Transaction {
            entries: vec![
                Entry {
                    account: "Account1".to_string(),
                    cents: 100,
                },
                Entry {
                    account: "Account2".to_string(),
                    cents: -200,
                },
            ],
            ..Default::default()
        });
        let mut blank = Some(line);
        let mut journal: Vec<Transaction> = Vec::new();

        finalize_transaction(&mut transaction, &mut blank, &mut journal);

        assert_eq!(journal.len(), 1);
        let journal_entry = &journal[0];
        assert_eq!(journal_entry.entries.len(), 3);
        assert_eq!(journal_entry.total_cents(), 0);
        assert_eq!(journal_entry.entries.last().unwrap().cents, 100);
    }

    #[test]
    fn test_move_transaction_no_blank_line() {
        let mut transaction = Some(Transaction {
            entries: vec![
                Entry {
                    account: "Account1".to_string(),
                    cents: 100,
                },
                Entry {
                    account: "Account2".to_string(),
                    cents: -100,
                },
            ],
            ..Default::default()
        });
        let mut blank: Option<Line> = None;
        let mut journal: Vec<Transaction> = Vec::new();

        finalize_transaction(&mut transaction, &mut blank, &mut journal);

        assert_eq!(journal.len(), 1);
        let journal_entry = &journal[0];
        assert_eq!(journal_entry.entries.len(), 2);
        assert_eq!(journal_entry.total_cents(), 0);
    }

    #[test]
    #[should_panic(expected = "Unbalanced transaction: 1970-01-01 Description\n    Account1    $1.00\n    Account2    $-2.00")]
    fn test_move_transaction_unbalanced_transaction() {
        let mut transaction = Some(Transaction {
            entries: vec![
                Entry {
                    account: "Account1".to_string(),
                    cents: 100,
                },
                Entry {
                    account: "Account2".to_string(),
                    cents: -200,
                },
            ],
            description: "Description".to_string(),
            ..Default::default()
        });
        let mut blank: Option<Line> = None;
        let mut journal: Vec<Transaction> = Vec::new();

        finalize_transaction(&mut transaction, &mut blank, &mut journal)
    }

    /*  Green light, code affirmed
        In woven tests, a new thread
        Peaceful mind now earned

        - a haiku by ChatGPT 4
     */
}
