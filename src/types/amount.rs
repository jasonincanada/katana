use std::fmt;
use super::Units;

// a generic amount of something
#[derive(Clone, Debug, PartialEq)]
pub enum AmountType {
    // an integer number of smallest divisible units of the commodity
    // and a number of decimal places after the unit place value
    // so for $10.25: Discrete(1025, 2)
    Discrete(i64, usize),

    Float(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Amount {
    pub units: Units,
    pub amount: AmountType
}
    
impl Amount {
    pub fn from(units: String, amount: f64) -> Self {

        // special case for the $ sign which we know divides into 100 cents
        let amount = if units == "$" {
            AmountType::Discrete((amount * 100.0).round() as i64, 2)
        } else {
            AmountType::Float(amount)
        };

        Amount {
            units,
            amount,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self.amount {
            AmountType::Discrete(amt, _) => amt == 0,
            AmountType::Float(amt) => amt == 0.0,
        }
    }

    pub fn negate(self) -> Amount {
        let negated = match self.amount {
            AmountType::Discrete(amt, dec) => AmountType::Discrete(-amt, dec),
            AmountType::Float(amt)         => AmountType::Float(-amt),
        };
        Amount {
            units: self.units,
            amount: negated,
        }
    }

    pub fn add(&mut self, other: &Self) {
        if self.units != other.units {
            panic!("Cannot add two amounts with different units")
        }

        match (&self.amount, &other.amount) {
            (AmountType::Discrete(l, d1), AmountType::Discrete(r, d2)) => {
                if d1 != d2 {
                    unimplemented!("Cannot add two discrete amounts with different decimal places")
                }
                self.amount = AmountType::Discrete(l+r, *d1);
            },
            (AmountType::Float(l), AmountType::Float(r)) => {
                self.amount = AmountType::Float(l+r);
            },
            (AmountType::Discrete(_, _), AmountType::Float(_)) =>
                panic!("Cannot add a discrete amount to a float amount"),
            (AmountType::Float(_), AmountType::Discrete(_, _)) =>
                panic!("Cannot add a float amount to a discrete amount")
        }
    }
}

impl fmt::Display for Amount {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: units
        match self.amount {
            AmountType::Discrete(amt, _) => write!(f, "${:.2}", amt as f64 / 100.0),
            AmountType::Float(amt)       => write!(f, "{:.3}", amt as f64),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let amount1 = Amount::from("$".to_string(), 10.25);
        assert_eq!(amount1, Amount {
            units: "$".to_string(),
            amount: AmountType::Discrete(1025, 2),
        });

        let amount2 = Amount::from("kg".to_string(), 2.5);
        assert_eq!(amount2, Amount {
            units: "kg".to_string(),
            amount: AmountType::Float(2.5),
        });
    }

    #[test]
    fn test_is_zero() {
        let amount1 = Amount::from("$".to_string(), 0.0);
        let amount2 = Amount::from("kg".to_string(), 0.0);
        assert!(amount1.is_zero());
        assert!(amount2.is_zero());
    }

    #[test]
    fn test_negate() {
        let amount1 = Amount::from("$".to_string(), 10.25);
        let amount2 = Amount::from("kg".to_string(), 2.5);
        assert_eq!(amount1.negate(), Amount {
            units: "$".to_string(),
            amount: AmountType::Discrete(-1025, 2),
        });
        assert_eq!(amount2.negate(), Amount {
            units: "kg".to_string(),
            amount: AmountType::Float(-2.5),
        });
    }

    #[test]
    fn test_add_same_units() {
        let mut amount1 = Amount::from("$".to_string(), 10.25);
        let amount2 = Amount::from("$".to_string(), 5.25);
        amount1.add(&amount2);
        assert_eq!(amount1, Amount {
            units: "$".to_string(),
            amount: AmountType::Discrete(1550, 2),
        });

        let mut amount3 = Amount::from("kg".to_string(), 2.5);
        let amount4 = Amount::from("kg".to_string(), 1.5);
        amount3.add(&amount4);
        assert_eq!(amount3, Amount {
            units: "kg".to_string(),
            amount: AmountType::Float(4.0),
        });
    }

    #[test]
    #[should_panic(expected = "Cannot add two amounts with different units")]
    fn test_add_different_units() {
        let mut amount1 = Amount::from("$".to_string(), 10.25);
        let amount2 = Amount::from("kg".to_string(), 2.5);
        amount1.add(&amount2);
    }
}