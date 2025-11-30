/// Result Formatter
/// 
/// Pretty-prints query results in table format

use crate::types::Value;
use crate::vm::executor::QueryResult;
use prettytable::{Table, Row, Cell, format};

/// Result formatter
pub struct Formatter;

impl Formatter {
    /// Format a query result as a pretty table
    pub fn format_result(result: &QueryResult, column_names: &[String]) -> String {
        if result.rows.is_empty() {
            if result.rows_affected > 0 {
                return format!("{} row(s) affected.", result.rows_affected);
            } else {
                return "No rows returned.".to_string();
            }
        }
        
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        
        // Add header
        let header_cells: Vec<Cell> = column_names
            .iter()
            .map(|name| Cell::new(name).style_spec("Fb"))
            .collect();
        table.add_row(Row::new(header_cells));
        
        // Add rows
        for row in &result.rows {
            let cells: Vec<Cell> = row
                .iter()
                .map(|value| Cell::new(&Self::format_value(value)))
                .collect();
            table.add_row(Row::new(cells));
        }
        
        format!("{}\n{} row(s) returned.", table.to_string(), result.rows.len())
    }
    
    /// Format a single value for display
    pub fn format_value(value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Real(f) => format!("{:.6}", f).trim_end_matches('0').trim_end_matches('.').to_string(),
            Value::Text(s) => s.clone(),
            Value::Blob(b) => format!("<BLOB {} bytes>", b.len()),
        }
    }
    
    /// Format an error message
    pub fn format_error(error: &str) -> String {
        format!("Error: {}", error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_value() {
        assert_eq!(Formatter::format_value(&Value::Null), "NULL");
        assert_eq!(Formatter::format_value(&Value::Integer(42)), "42");
        assert_eq!(Formatter::format_value(&Value::Real(3.14159)), "3.14159");
        assert_eq!(Formatter::format_value(&Value::Text("hello".to_string())), "hello");
        assert_eq!(Formatter::format_value(&Value::Blob(vec![1, 2, 3])), "<BLOB 3 bytes>");
    }
    
    #[test]
    fn test_format_empty_result() {
        let result = QueryResult::new();
        let output = Formatter::format_result(&result, &[]);
        assert_eq!(output, "No rows returned.");
    }
    
    #[test]
    fn test_format_affected_rows() {
        let result = QueryResult::with_affected(5);
        let output = Formatter::format_result(&result, &[]);
        assert_eq!(output, "5 row(s) affected.");
    }
}

