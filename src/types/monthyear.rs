use std::fmt;
use chrono::{NaiveDate, Datelike};


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd)]
pub struct MonthYear {
    pub month: u32,
    pub year : u32,
}

impl MonthYear {
    pub fn new(month: u32, year: u32) -> Self {
        if (1..=12).contains(&month) {
            Self { month, year }
        } else {
            panic!("Invalid month: {}", month);
        }
    }

    pub fn next_month(&self) -> Self {
        let (new_month, new_year) = if self.month == 12 {
            (1, self.year + 1)
        } else {
            (self.month + 1, self.year)
        };
        Self {
            month: new_month,
            year: new_year
        }
    }

    pub fn from_naivedate(date: NaiveDate) -> MonthYear {
        Self {
            month: date.month(),
            year: date.year() as u32
        }
    }
}

impl fmt::Display for MonthYear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:02}", self.year, self.month)
    }
}


#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use super::MonthYear;

    #[test]
    fn test_new() {
        let month_year = MonthYear::new(5, 2022);
        assert_eq!(month_year.month, 5);
        assert_eq!(month_year.year, 2022);
    }

    #[test]
    #[should_panic(expected = "Invalid month: 13")]
    fn test_new_invalid_month() {
        MonthYear::new(13, 2022);
    }

    #[test]
    fn test_next_month() {
        let month_year = MonthYear::new(5, 2022);
        let next_month_year = month_year.next_month();
        assert_eq!(next_month_year.month, 6);
        assert_eq!(next_month_year.year, 2022);
    }

    #[test]
    fn test_next_month_wrap() {
        let month_year = MonthYear::new(12, 2022);
        let next_month_year = month_year.next_month();
        assert_eq!(next_month_year.month, 1);
        assert_eq!(next_month_year.year, 2023);
    }

    #[test]
    fn test_from_naivedate() {
        let naive_date = NaiveDate::from_ymd_opt(2022, 5, 15).unwrap();
        let month_year = MonthYear::from_naivedate(naive_date);
        assert_eq!(month_year.month, 5);
        assert_eq!(month_year.year, 2022);
    }
}
