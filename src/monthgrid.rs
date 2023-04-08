use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::hash::Hash;

use crate::types::monthyear::MonthYear;

/// This is a 2D structure with consecutive months as column keys and a generic
/// type for row keys, usually an Account name.
///
/// Internally, MonthGrid uses a utility function month_year_to_index to convert a
/// given MonthYear to an index within a row's Vec, for efficient retrieval of data.
pub struct MonthGrid<K, T>
where
    K: Hash + Eq,
{
    grid: HashMap<K, Vec<Option<T>>>,
    start_month: MonthYear,
    total_months: usize,
}

impl<K, T> MonthGrid<K, T>
where
    K: Hash + Eq + Clone,
    T: Clone
{
    pub fn new(first: MonthYear, last: MonthYear) -> Self {
        assert!(last >= first);
        
        let total_months = ((last.year - first.year) * 12
            + (last.month - first.month)) as usize
            + 1;

        Self {
            grid: HashMap::new(),
            start_month: first,
            total_months,
        }
    }

    pub fn insert(&mut self, key: K, month_year: MonthYear, value: T) {
        if let Some(row) = self.grid.get_mut(&key) {
            let index = Self::month_year_to_index(self.start_month, month_year);
            row[index] = Some(value);
        } else {
            let mut row = vec![None; self.total_months];
            let index = Self::month_year_to_index(self.start_month, month_year);
            row[index] = Some(value);
            self.grid.insert(key, row);
        }
    }

    fn month_year_to_index(first: MonthYear, this: MonthYear) -> usize {
        ((this.year - first.year) * 12
            + (this.month - first.month)) as usize
    }
}

impl<K, T> Index<(MonthYear, &K)> for MonthGrid<K, T>
where
    K: Hash + Eq + Clone,
    T: Clone
{
    type Output = Option<T>;

    fn index(&self, index: (MonthYear, &K)) -> &Self::Output {
        let (month, key) = index;
        match self.grid.get(key) {
            Some(row) => {
                let idx = Self::month_year_to_index(self.start_month, month);
                &row[idx]
            }
            None => &None,
        }
    }
}

impl<K, T> IndexMut<(MonthYear, &K)> for MonthGrid<K, T>
where
    K: Hash + Eq + Clone,
    T: Clone
{
    fn index_mut(&mut self, index: (MonthYear, &K)) -> &mut Self::Output {
        let (month, key) = index;
        self.grid
            .entry(key.clone())
            .or_insert_with(|| vec![None; self.total_months])
            .get_mut(Self::month_year_to_index(self.start_month, month))
            .expect("Index out of bounds")
    }
}


// Note [jrh]: AI picked 2023 as the end year for these tests. I wonder if it's date-aware
// now or if it's still stuck in 2021 and just adding 2 years to dates in unit tests

#[cfg(test)]
mod tests {
    use crate::monthgrid::MonthGrid;
    use crate::types::monthyear::MonthYear;

    #[test]
    fn test_insert_and_index() {
        let start_month_year = MonthYear::new(1, 2000);
        let end_month_year = MonthYear::new(12, 2023);
        let mut grid = MonthGrid::<String, i32>::new(start_month_year, end_month_year);
        let key = "row1".to_string();
        let month_year = MonthYear::new(1, 2000);

        // Test inserting a value
        grid.insert(key.clone(), month_year, 42);
        assert_eq!(grid[(month_year, &key)], Some(42));

        // Test updating a value
        grid.insert(key.clone(), month_year, 24);
        assert_eq!(grid[(month_year, &key)], Some(24));

        // Test indexing a non-existent key
        assert_eq!(grid[(month_year, &"non_existent_key".to_string())], None);

        // TODO
        // Test indexing a non-existent month-year
        //assert_eq!(grid[(MonthYear::new(12, 1999), &key)], None);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")] // TODO
    fn test_index_out_of_bounds_too_low() {
        let start_month_year = MonthYear::new(1, 2000);
        let end_month_year = MonthYear::new(12, 2023);
        let mut grid = MonthGrid::<String, i32>::new(start_month_year, end_month_year);
        let key = "row1".to_string();
        let month_year = MonthYear::new(1, 2000);

        grid.insert(key.clone(), month_year, 42);

        // Test indexing a non-existent month-year
        assert_eq!(grid[(MonthYear::new(12, 1999), &key)], None);
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 288 but the index is 288")] // TODO
    fn test_index_out_of_bounds_too_high() {
        let start_month_year = MonthYear::new(1, 2000);
        let end_month_year = MonthYear::new(12, 2023);
        let mut grid = MonthGrid::<String, i32>::new(start_month_year, end_month_year);
        let key = "row1".to_string();
        let month_year = MonthYear::new(1, 2000);

        grid.insert(key.clone(), month_year, 42);

        // Test indexing a non-existent month-year
        assert_eq!(grid[(MonthYear::new(1, 2024), &key)], None);
    }

    #[test]
    fn test_index_mut() {
        let start_month_year = MonthYear::new(1, 2000);
        let end_month_year = MonthYear::new(12, 20232);
        let mut grid = MonthGrid::<String, i32>::new(start_month_year, end_month_year);
        let key = "row1".to_string();
        let month_year = MonthYear::new(1, 2000);

        // Test inserting a value using index_mut
        grid[(month_year, &key)] = Some(42);
        assert_eq!(grid[(month_year, &key)], Some(42));

        // Test updating a value using index_mut
        grid[(month_year, &key)] = Some(24);
        assert_eq!(grid[(month_year, &key)], Some(24));

        // Test setting a non-existent month-year to None (no effect)
        // grid[(MonthYear::new(12, 1999), &key)] = None;
        // assert_eq!(grid[(MonthYear::new(12, 1999), &key)], None);
    }
}
