/// Logical Plan - High-level query representation
/// 
/// Represents the logical structure of a query without physical implementation details

use crate::sql::ast::{Expr, OrderBy};

/// Logical plan node
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalPlan {
    /// Table scan
    Scan {
        table: String,
        alias: Option<String>,
    },
    
    /// Filter (WHERE clause)
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
    
    /// Projection (SELECT columns)
    Projection {
        input: Box<LogicalPlan>,
        expressions: Vec<ProjectionExpr>,
    },
    
    /// Sort (ORDER BY)
    Sort {
        input: Box<LogicalPlan>,
        order_by: Vec<OrderBy>,
    },
    
    /// Limit
    Limit {
        input: Box<LogicalPlan>,
        limit: usize,
        offset: Option<usize>,
    },
    
    /// Insert
    Insert {
        table: String,
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expr>>,
    },
    
    /// Update
    Update {
        table: String,
        assignments: Vec<(String, Expr)>,
        filter: Option<Expr>,
    },
    
    /// Delete
    Delete {
        table: String,
        filter: Option<Expr>,
    },
    
    /// Create Table
    CreateTable {
        table: String,
        columns: Vec<ColumnSpec>,
    },
}

/// Projection expression with optional alias
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionExpr {
    pub expr: Expr,
    pub alias: Option<String>,
}

/// Column specification for CREATE TABLE
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSpec {
    pub name: String,
    pub data_type: DataType,
    pub not_null: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub default: Option<String>,
}

/// Data type specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    Integer,
    Real,
    Text,
    Blob,
}

impl LogicalPlan {
    /// Get the input plan (for plans that have one)
    pub fn input(&self) -> Option<&LogicalPlan> {
        match self {
            LogicalPlan::Filter { input, .. }
            | LogicalPlan::Projection { input, .. }
            | LogicalPlan::Sort { input, .. }
            | LogicalPlan::Limit { input, .. } => Some(input),
            _ => None,
        }
    }
    
    /// Get a mutable reference to the input plan
    pub fn input_mut(&mut self) -> Option<&mut LogicalPlan> {
        match self {
            LogicalPlan::Filter { input, .. }
            | LogicalPlan::Projection { input, .. }
            | LogicalPlan::Sort { input, .. }
            | LogicalPlan::Limit { input, .. } => Some(input),
            _ => None,
        }
    }
}

impl std::fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalPlan::Scan { table, alias } => {
                if let Some(alias) = alias {
                    write!(f, "Scan: {} AS {}", table, alias)
                } else {
                    write!(f, "Scan: {}", table)
                }
            }
            LogicalPlan::Filter { input, .. } => {
                write!(f, "Filter\n  {}", input)
            }
            LogicalPlan::Projection { input, expressions } => {
                write!(f, "Project: {} columns\n  {}", expressions.len(), input)
            }
            LogicalPlan::Sort { input, order_by } => {
                write!(f, "Sort: {} columns\n  {}", order_by.len(), input)
            }
            LogicalPlan::Limit { input, limit, offset } => {
                if let Some(offset) = offset {
                    write!(f, "Limit: {} OFFSET {}\n  {}", limit, offset, input)
                } else {
                    write!(f, "Limit: {}\n  {}", limit, input)
                }
            }
            LogicalPlan::Insert { table, .. } => {
                write!(f, "Insert: {}", table)
            }
            LogicalPlan::Update { table, .. } => {
                write!(f, "Update: {}", table)
            }
            LogicalPlan::Delete { table, .. } => {
                write!(f, "Delete: {}", table)
            }
            LogicalPlan::CreateTable { table, columns } => {
                write!(f, "CreateTable: {} ({} columns)", table, columns.len())
            }
        }
    }
}

