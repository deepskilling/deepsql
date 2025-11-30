/// LIMIT/OFFSET execution
/// 
/// Implements result limiting and offsetting

use crate::error::Result;
use crate::types::Value;

/// LIMIT/OFFSET executor
pub struct LimitExecutor;

impl LimitExecutor {
    /// Apply LIMIT and OFFSET to rows
    pub fn apply(
        rows: Vec<Vec<Value>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Vec<Value>>> {
        let mut result = rows;
        
        // Apply OFFSET first
        if let Some(offset_val) = offset {
            if offset_val > 0 {
                if offset_val >= result.len() {
                    result.clear();
                } else {
                    result.drain(0..offset_val);
                }
            }
        }
        
        // Then apply LIMIT
        if let Some(limit_val) = limit {
            if result.len() > limit_val {
                result.truncate(limit_val);
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_limit_only() {
        let rows = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
        ];
        
        let result = LimitExecutor::apply(rows, Some(2), None).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], Value::Integer(1));
        assert_eq!(result[1][0], Value::Integer(2));
    }
    
    #[test]
    fn test_offset_only() {
        let rows = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
        ];
        
        let result = LimitExecutor::apply(rows, None, Some(2)).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], Value::Integer(3));
        assert_eq!(result[1][0], Value::Integer(4));
    }
    
    #[test]
    fn test_limit_and_offset() {
        let rows = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
            vec![Value::Integer(5)],
        ];
        
        let result = LimitExecutor::apply(rows, Some(2), Some(1)).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], Value::Integer(2));
        assert_eq!(result[1][0], Value::Integer(3));
    }
}

