/// Journal types

use std::str::FromStr;
use crate::common::is_all_whitespace;
use crate::types::Account;


/* Amount */

// the two types of input on the right side of an entry line
#[derive(Clone, Debug, PartialEq)]
pub enum Amount { // TODO: rename to LineAmount
    Cents(i64),
    Blank
}

impl FromStr for Amount {
    type Err = AmountParseErrors;

    // parse("$1.25")  -> Amount::Cents(125)
    // parse("$-1.25") -> Amount::Cents(-125)
    // parse("")       -> Amount::Blank
    // parse(" ")      -> Amount::Blank
    fn from_str(amount: &str) -> Result<Self, Self::Err> {

        if let Some(amount) = amount.strip_prefix('$') {
            if let Ok(cents) = amount.parse::<f64>() {
                return Ok(Amount::Cents((cents * 100.0).round() as i64))
            }
        }

        if is_all_whitespace(amount) {
            return Ok(Amount::Blank)
        }

        Err(AmountParseErrors::InvalidAmount)
    }
}

impl Amount {
    pub fn is_blank(&self) -> bool {
        matches!(self, Amount::Blank)
    }
}

#[derive(Debug, PartialEq)]
pub enum AmountParseErrors {
    InvalidAmount
}


/* Line */

// an account line from the journal text file, with an optional dollar amount
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub account: Account,
    pub amount : Amount
}

impl FromStr for Line {
    type Err = LineParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut parts = line.split_whitespace();

        let account = parts.next()
                           .ok_or(LineParseError::MissingAccount) ?
                           .to_string();

        let amount  = parts.next()
                           .map_or(Ok(Amount::Blank), |part| {
                               Amount::from_str(part).map_err(|_| LineParseError::InvalidAmount)
                           })?;

        Ok(Line { account, amount })
    }
}

#[derive(Debug, PartialEq)]
pub enum LineParseError {
    MissingAccount,
    InvalidAmount,
}


/* Tests */

#[cfg(test)]
mod tests {
    use super::{Amount, AmountParseErrors, FromStr, Line, LineParseError};

    #[test] fn amount_parse_positive_amount()   { assert_eq!(Amount::from_str("$1.25"), Ok(Amount::Cents(125))); }
    #[test] fn amount_parse_negative_amount()   { assert_eq!(Amount::from_str("$-1.25"), Ok(Amount::Cents(-125))); }
    #[test] fn amount_parse_empty_string()      { assert_eq!(Amount::from_str(""), Ok(Amount::Blank)); }
    #[test] fn amount_parse_whitespace_string() { assert_eq!(Amount::from_str(" "), Ok(Amount::Blank)); }
    #[test] fn amount_parse_zero_amount()       { assert_eq!(Amount::from_str("$0.00"), Ok(Amount::Cents(0))); }
    #[test] fn amount_parse_large_amount()      { assert_eq!(Amount::from_str("$12345.67"), Ok(Amount::Cents(1234567))); }

    #[test]
    fn amount_parse_invalid_string() {
        assert_eq!(Amount::from_str("foo"), Err(AmountParseErrors::InvalidAmount));
    }

    #[test]
    fn test_is_blank() {
        assert!(Amount::Blank.is_blank());
        assert!(!Amount::Cents(10).is_blank());
    }

    #[test]
    fn test_parse_line() {
        
        // blank line
        assert_eq!(Line::from_str(""), Err(LineParseError::MissingAccount));

        // blank amount
        assert_eq!(Line::from_str("acct:sub-acct"),
                   Ok(Line { account: "acct:sub-acct".to_owned(),
                             amount : Amount::Blank
                           }));

        assert_eq!(Line::from_str("acct:sub-acct "),
                   Ok(Line { account: "acct:sub-acct".to_owned(),
                             amount : Amount::Blank
                           }));

        assert_eq!(Line::from_str("acct:sub-acct             "),
                   Ok(Line { account: "acct:sub-acct".to_owned(),
                             amount : Amount::Blank
                           }));

        // an actual amount in dollars/cents
        assert_eq!(Line::from_str("expenses:food:tim-hortons $-1.25"),
                   Ok(Line { account: "expenses:food:tim-hortons".to_owned(),
                             amount : Amount::Cents(-125)
                           }));

        // multiple spaces between the two sides
        assert_eq!(Line::from_str("expenses:food:tim-hortons \t   $-1.25"),
                   Ok(Line { account: "expenses:food:tim-hortons".to_owned(),
                             amount : Amount::Cents(-125)
                           }));
    }
}
