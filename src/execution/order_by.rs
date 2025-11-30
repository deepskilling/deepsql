/// ORDER BY execution
/// 
/// Implements result sorting

use crate::error::Result;
use crate::sql::ast::{OrderBy, OrderDirection};
use crate::types::Value;

/// ORDER BY executor
pub struct OrderByExecutor;

impl OrderByExecutor {
    /// Sort rows by ORDER BY clause
    pub fn sort(_rows: &mut [Vec<Value>], _order_by: &[OrderBy]) -> Result<()> {
        // TODO: Implement full ORDER BY sorting
        // For now, keep rows as-is
        Ok(())
    }
    
    /// Sort rows by a single column
    pub fn sort_by_column(
        rows: &mut [Vec<Value>],
        column_index: usize,
        direction: OrderDirection,
    ) -> Result<()> {
        rows.sort_by(|a, b| {
            if column_index >= a.len() || column_index >= b.len() {
                return std::cmp::Ordering::Equal;
            }
            
            let ordering = a[column_index].compare(&b[column_index]).unwrap_or(std::cmp::Ordering::Equal);
            
            match direction {
                OrderDirection::Asc => ordering,
                OrderDirection::Desc => ordering.reverse(),
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sort_by_column_asc() {
        let mut rows = vec![
            vec![Value::Integer(3)],
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ];
        
        OrderByExecutor::sort_by_column(&mut rows, 0, OrderDirection::Asc).unwrap();
        
        assert_eq!(rows[0][0], Value::Integer(1));
        assert_eq!(rows[1][0], Value::Integer(2));
        assert_eq!(rows[2][0], Value::Integer(3));
    }
    
    #[test]
    fn test_sort_by_column_desc() {
        let mut rows = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(3)],
            vec![Value::Integer(2)],
        ];
        
        OrderByExecutor::sort_by_column(&mut rows, 0, OrderDirection::Desc).unwrap();
        
        assert_eq!(rows[0][0], Value::Integer(3));
        assert_eq!(rows[1][0], Value::Integer(2));
        assert_eq!(rows[2][0], Value::Integer(1));
    }
}

