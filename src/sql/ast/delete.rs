/// DELETE statement AST

use super::expr::Expr;

/// DELETE statement structure
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    /// Table name
    pub table: String,
    
    /// WHERE clause
    pub where_clause: Option<Expr>,
}

impl DeleteStatement {
    /// Create a new DELETE statement for a table
    pub fn new(table: String) -> Self {
        DeleteStatement {
            table,
            where_clause: None,
        }
    }
}

