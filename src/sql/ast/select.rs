/// SELECT statement AST

use super::expr::{Expr, OrderBy};

/// SELECT statement structure
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    /// DISTINCT flag
    pub distinct: bool,
    
    /// SELECT columns (* or list of expressions)
    pub columns: Vec<SelectColumn>,
    
    /// FROM table name
    pub from: Option<String>,
    
    /// WHERE clause
    pub where_clause: Option<Expr>,
    
    /// ORDER BY clause
    pub order_by: Vec<OrderBy>,
    
    /// LIMIT clause
    pub limit: Option<usize>,
    
    /// OFFSET clause
    pub offset: Option<usize>,
}

/// SELECT column specification
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum SelectColumn {
    /// SELECT * (all columns)
    Star,
    
    /// SELECT expr [AS alias]
    Expr {
        expr: Expr,
        alias: Option<String>,
    },
}

impl SelectStatement {
    /// Create a new empty SELECT statement
    pub fn new() -> Self {
        SelectStatement {
            distinct: false,
            columns: Vec::new(),
            from: None,
            where_clause: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }
}

impl Default for SelectStatement {
    fn default() -> Self {
        Self::new()
    }
}

