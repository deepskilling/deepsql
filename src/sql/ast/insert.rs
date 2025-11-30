/// INSERT statement AST

use super::expr::Expr;

/// INSERT statement structure
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    /// Table name
    pub table: String,
    
    /// Column names (optional)
    pub columns: Option<Vec<String>>,
    
    /// Values to insert
    pub values: Vec<Vec<Expr>>,
}

impl InsertStatement {
    /// Create a new INSERT statement for a table
    pub fn new(table: String) -> Self {
        InsertStatement {
            table,
            columns: None,
            values: Vec::new(),
        }
    }
}

