/// UPDATE statement AST

use super::expr::Expr;

/// UPDATE statement structure
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    /// Table name
    pub table: String,
    
    /// SET column = value pairs
    pub assignments: Vec<Assignment>,
    
    /// WHERE clause
    pub where_clause: Option<Expr>,
}

/// Column assignment (column = value)
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    /// Column name
    pub column: String,
    
    /// New value expression
    pub value: Expr,
}

impl UpdateStatement {
    /// Create a new UPDATE statement for a table
    pub fn new(table: String) -> Self {
        UpdateStatement {
            table,
            assignments: Vec::new(),
            where_clause: None,
        }
    }
}

