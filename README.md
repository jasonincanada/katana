# katana

This is a Rust program that runs reports on double-entry ledgers stored as plain text files. It is in early stages of development and currently only supports the **register** report.  **katana** is largely modeled after https://hledger.org/. I've used hledger for three years and finally took on the challenge of writing a similar program in Rust. It will set itself apart from the other utilities once the terminal UI is underway, but for now it does basic report generation to stdout based on an input ledger file.

This is my first programming project where I'm heavily assisted by ChatGPT. Its refactoring and code-writing help is already so pervasive that I made the decision not to leave comments in the code where and how I was assisted. Instead, I have listed OpenAI as a co-author in the `authors` section of the project's `Cargo.toml` file. Otherwise I have not annotated specifically where and how the AI helped. I should have much more to say about this interesting new development as the project goes on since AI is quickly becoming an integral component of my dev process, both at work and in these hobby projects.


## Journal file

katana reads plaintext ledger files in the same format as hledger and similar programs. Here are five example transactions:

```text
2023/03/01 opening balances
    assets:cash                  $50
    assets:savings               $1000.00
    equity:opening-balances

2023/03/01 YouTube Premium
    expenses:entertainment:youtube          $13.55
    credit:visa                            $-13.55

2023/03/03 AWS
    expenses:internet:aws           $12.35
    credit:visa

2023/03/07 Payroll Deposit
    assets:savings               $2000
    income:payroll:workplace    $-2000

2023/03/07 Sandwich, Chili
    assets:savings                    $-14.99
    expenses:tips                       $0.50
    assets:cash                        $-0.50
    expenses:food:tim-hortons
```

Note that a single entry in any transaction can be blank and katana balances the transaction to zero for you by calculating the sum of the other amounts and taking the negation of it.


## Reports

This is an example of the **register** report, which shows each posting to an account and a cummulative balance.

```text
Have 5 transactions
Register report for account assets:savings:
2023/03/01 opening balances               assets:savings                   $1000.00   $1000.00
2023/03/07 Payroll Deposit                assets:savings                   $2000.00   $3000.00
2023/03/07 Sandwich, Chili                assets:savings                    $-14.99   $2985.01
```


## Roadmap

- [x] Read in ledger files with dollar/cents amounts
    - [ ] General units, not just dollars
- [x] Reports
    - [x] **register** - One line per post to a single specified account, with a cummulative total
        - [ ] Any number of specified accounts
    - [ ] **balance** - Show balance changes to an account(s) grouped by time period (weekly, monthly)
- [ ] Stream the file instead of reading it into a big string
- [ ] Tiny Docker image
- [ ] Slick UI for adding new transactions with the minimal number of keystrokes. Full color terminal app based on ncurses


## About the Name

**Me:** Is Katana an appropriate name for a personal finance management program?

**ChatGPT:** While "Katana" might not be an immediately recognizable name for a personal finance management program, it could still work if you can find a way to connect the name to the purpose of your software. Katana is a type of Japanese sword known for its precision and sharpness. If you can draw parallels between the qualities of a katana and the features of your personal finance management program, such as accuracy, efficiency, and cutting-edge technology, it could be an interesting and memorable choice.
