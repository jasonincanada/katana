/// Journal types

use lazy_static::lazy_static;
use std::str::FromStr;
use regex::Regex;

use crate::common::is_all_whitespace;
use crate::types::{Account, Amount, Units};


// the two types of input on the right side of an entry line
#[derive(Clone, Debug, PartialEq)]
pub enum LineAmount {
    Amount(Amount),
    Blank
}


/* Line */

// an account line from the journal text file, with an optional dollar amount
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub account: Account,
    pub amount : LineAmount
}

#[derive(Debug, PartialEq)]
enum ParsedLine {
    AccountWithAmount(Account, Units, f64),
    AccountOnly(Account),
    Invalid
}

lazy_static! {
    static ref ACCOUNT_AND_AMOUNT_REGEX: Regex =
        Regex::new(r"(?x)
            (?P<account>[[:alnum:]:-]+)
            (?:
                \s\s+
                (?P<units>[a-zA-Z\$]+)
                \s*
                (?P<amount>[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)
              |
                \s\s+
                (?P<amount2>[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)
                \s*
                (?P<units2>[a-zA-Z\$]+)
            )
        ").unwrap();

    static ref ACCOUNT_ONLY_REGEX: Regex = 
        Regex::new(r"^\s*(?P<account>[[:alnum:]:-]+)\s*$").unwrap();
}

fn parse_account_and_amount(input: &str) -> ParsedLine {
    if let Some(captures) = ACCOUNT_AND_AMOUNT_REGEX.captures(input) {
        let account = captures.name("account").unwrap().as_str().to_string();
        let units = captures.name("units").or_else(|| captures.name("units2")).unwrap().as_str().to_string();
        let amount_str = captures.name("amount").or_else(|| captures.name("amount2")).unwrap().as_str();
        let amount = f64::from_str(amount_str).unwrap();
        ParsedLine::AccountWithAmount(account, units, amount)
    } else if let Some(account) = parse_account_only(input) {
        ParsedLine::AccountOnly(account)
    } else {
        ParsedLine::Invalid
    }
}

fn parse_account_only(input: &str) -> Option<String> {
    if let Some(captures) = ACCOUNT_ONLY_REGEX.captures(input) {
        let account = captures.name("account").unwrap().as_str().to_string();
        Some(account)
    } else {
        None
    }
}

impl FromStr for Line {
    type Err = LineParseError;

    // expenses:food:tim-hortons  $1.62
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if is_all_whitespace(line) {
            return Err(LineParseError::MissingAccount)
        }
        match parse_account_and_amount(line) {
            ParsedLine::AccountWithAmount(account, units, amount) => {
                Ok(Line {
                    account,
                    amount: LineAmount::Amount(Amount::from(units, amount))
                })
            },
            ParsedLine::AccountOnly(account) => {
                Ok(Line {
                    account,
                    amount: LineAmount::Blank
                })
            },
            ParsedLine::Invalid => Err(LineParseError::Unknown),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LineParseError {
    MissingAccount,
    Unknown,
}


/* Tests */

#[cfg(test)]
mod tests {
    use crate::types::{Amount, AmountType};
    use crate::journal::types::{parse_account_and_amount, ParsedLine, LineParseError};
    use super::{LineAmount, FromStr, Line};

    #[test]
    fn test_parse_line() {
        
        // blank line
        assert_eq!(Line::from_str(""), Err(LineParseError::MissingAccount));

        // blank amount
        assert_eq!(Line::from_str("acct:sub-acct"),
                   Ok(Line { account: "acct:sub-acct".to_owned(),
                             amount : LineAmount::Blank
                           }));

        assert_eq!(Line::from_str("acct:sub-acct "),
                   Ok(Line { account: "acct:sub-acct".to_owned(),
                             amount : LineAmount::Blank
                           }));

        assert_eq!(Line::from_str("acct:sub-acct             "),
                   Ok(Line { account: "acct:sub-acct".to_owned(),
                             amount : LineAmount::Blank
                           }));

        // an actual amount in dollars/cents
        assert_eq!(Line::from_str("expenses:food:tim-hortons  $-1.25"),
                   Ok(Line { account: "expenses:food:tim-hortons".to_owned(),
                             amount : LineAmount::Amount(Amount {
                                    units : "$".to_owned(),
                                    amount: AmountType::Discrete(-125, 2)
                            })}));

        // multiple whitespace between the two sides
        assert_eq!(Line::from_str("expenses:food:tim-hortons  \t  $-1.25"),
                   Ok(Line { account: "expenses:food:tim-hortons".to_owned(),
                             amount : LineAmount::Amount(Amount {
                                    units : "$".to_owned(),
                                    amount: AmountType::Discrete(-125, 2)
                            })}));
        

        assert_eq!(Line::from_str("usage:power  \t  308 kWh"),
                   Ok(Line { account: "usage:power".to_owned(),
                             amount : LineAmount::Amount(Amount {
                                    units:  "kWh".to_owned(),
                                    amount: AmountType::Float(308.0)
                            })}));
    }

    #[test]
    fn test_parse_account_amount() {
        let input = "acc123  100.5USD";
        let result = parse_account_and_amount(input);
        assert_eq!(result, ParsedLine::AccountWithAmount("acc123".to_owned(), "USD".to_owned(), 100.5));
    }

    #[test]
    fn test_parse_account_amount_needs_two_spaces_after_account() {
        let input = "acc123 100.5USD";
        let result = parse_account_and_amount(input);
        assert_eq!(result, ParsedLine::Invalid);
    }
    
    #[test]
    fn test_parse_account_amount_dollar_sign_right() {
        let input = "acc123  100.5$";
        let result = parse_account_and_amount(input);
        assert_eq!(result, ParsedLine::AccountWithAmount("acc123".to_owned(), "$".to_owned(), 100.5));
    }

    #[test]
    fn test_parse_account_amount_dollar_sign_left() {
        let input = "acc123  $100.5";
        let result = parse_account_and_amount(input);
        assert_eq!(result, ParsedLine::AccountWithAmount("acc123".to_owned(), "$".to_owned(), 100.5));
    }

    #[test]
    fn test_parse_account_amount_dollar_sign_left_with_space() {
        let input = "acc123  $ 100.5";
        let result = parse_account_and_amount(input);
        assert_eq!(result, ParsedLine::AccountWithAmount("acc123".to_owned(), "$".to_owned(), 100.5));
    }

    #[test]
    fn test_parse_account_amount_kwh() {
        let input = "usage:power  308 kWh";
        let result = parse_account_and_amount(input);
        assert_eq!(result, ParsedLine::AccountWithAmount("usage:power".to_owned(), "kWh".to_owned(), 308.0));
    }

    #[test]
    fn test_parse_account_amount_kwh_hyphen() {
        let input = "usage-power  kWh308";
        let result = parse_account_and_amount(input);
        assert_eq!(result, ParsedLine::AccountWithAmount("usage-power".to_owned(), "kWh".to_owned(), 308.0));
    }

}
