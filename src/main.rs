mod common;
mod journal;
mod reports;
mod transaction;
mod types;

use journal::Journal;
use reports::register_report;

fn main() {
    let ledger = r".\journal.txt";
    let contents = std::fs::read_to_string(ledger)
                            .expect("Couldn't read journal file");
    let journal = match Journal::from_lines(contents.lines()) {
        Ok(journal) => journal,
        Err(error) => panic!("Error reading journal: {}", error),
    };

    // $ katana register assets:savings
    let account = "assets:savings".to_owned();
    let report  = register_report(&journal,
                                  &account);

    println!("Have {} transactions", journal.transactions.len());
    println!("Register report for account {}:", account);

    for line in report {
        println!("{}", line);
    }
}
