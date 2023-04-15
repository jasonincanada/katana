mod common;
mod iterators;
mod journal;
mod monthgrid;
mod reports;
mod transaction;
mod types;

use clap::{App, Arg};
use std::fs;

use journal::Journal;
use monthgrid::MonthGrid;
use reports::balance::balance_changes;
use reports::register::register_report;
use types::{Account, amount::Amount, monthyear::MonthYear};

fn main() {
    let args = get_args();
    let journal_file = args.value_of("journal").expect("Journal file not specified");
    let journal = read_journal(journal_file);
    let report = args.value_of("report").unwrap();
    
    match report {
        "balance" => {
            let account = args.value_of("account")
                              .expect("Need an account name for the balance report");
            balance(&journal, account);
        },
        "register" => {
            let account = args.value_of("account")
                              .expect("Need an account name for the register report");
            register(&journal, account);
        },
        _ => panic!("Unknown report type"),
    }
}

// $ katana balance
fn balance(journal: &Journal, account: &str) {
    let account = account.to_string();
    let month: MonthYear = MonthYear::new(4, 2023);
    let report: MonthGrid<Account, Amount> = balance_changes(journal);

    println!("Balance changes for {} in {}: {:?}",
        account,
        month,
        report[(month, &account)]);
}

// $ katana register
fn register(journal: &Journal, account: &str) {
    let account = account.to_string();
    let report = register_report(journal, &account);

    println!("Register report for account {}:", account);
    for line in report {
        println!("{}", line);
    }
}

fn read_journal(journal_file: &str) -> Journal {
    let contents = fs::read_to_string(journal_file)
                      .expect("Couldn't read journal file");

    Journal::from_lines(contents.lines())
            .unwrap_or_else(|error| panic!("Error reading journal: {}", error))
}

fn get_args() -> clap::ArgMatches {
    App::new("katana")
        .arg(
            Arg::new("report")
                .help("The report to run")
                .index(1)
                .required(true)
                .possible_values(&["balance", "register"])
        )
        .arg(
            Arg::new("account")
                .short('a')
                .long("account")
                .value_name("ACCOUNT")
                .help("Set the account name")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("journal")
                .short('j')
                .long("journal")
                .value_name("JOURNAL")
                .help("Set the journal file")
                .takes_value(true)
                .required(true),
        )
        .get_matches()
}
