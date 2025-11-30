/// Physical Plan - Low-level execution plan
/// 
/// Represents the physical execution strategy with concrete operators

use crate::planner::logical::LogicalPlan;
use crate::sql::ast::{Expr, OrderBy};

/// Physical plan node (maps to VM opcodes)
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicalPlan {
    /// Table scan
    TableScan {
        table: String,
    },
    
    /// Index scan (future optimization)
    IndexScan {
        table: String,
        index: String,
    },
    
    /// Filter rows
    Filter {
        input: Box<PhysicalPlan>,
        predicate: Expr,
    },
    
    /// Project columns
    Project {
        input: Box<PhysicalPlan>,
        expressions: Vec<Expr>,
        aliases: Vec<Option<String>>,
    },
    
    /// Sort rows
    Sort {
        input: Box<PhysicalPlan>,
        order_by: Vec<OrderBy>,
    },
    
    /// Limit rows
    Limit {
        input: Box<PhysicalPlan>,
        limit: usize,
        offset: usize,
    },
    
    /// Insert rows
    Insert {
        table: String,
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expr>>,
    },
    
    /// Update rows
    Update {
        table: String,
        assignments: Vec<(String, Expr)>,
        filter: Option<Expr>,
    },
    
    /// Delete rows
    Delete {
        table: String,
        filter: Option<Expr>,
    },
}

impl PhysicalPlan {
    /// Convert logical plan to physical plan
    pub fn from_logical(logical: LogicalPlan) -> Self {
        match logical {
            LogicalPlan::CreateIndex { name, table, .. } => {
                PhysicalPlan::TableScan { table: format!("{}_{}", table, name) }
            }
            LogicalPlan::Transaction { .. } => {
                PhysicalPlan::TableScan { table: "transaction".into() }
            }
            LogicalPlan::Scan { table, .. } => {
                PhysicalPlan::TableScan { table }
            }
            
            LogicalPlan::Filter { input, predicate } => {
                PhysicalPlan::Filter {
                    input: Box::new(PhysicalPlan::from_logical(*input)),
                    predicate,
                }
            }
            
            LogicalPlan::Projection { input, expressions } => {
                let (exprs, aliases): (Vec<_>, Vec<_>) = expressions
                    .into_iter()
                    .map(|pe| (pe.expr, pe.alias))
                    .unzip();
                
                PhysicalPlan::Project {
                    input: Box::new(PhysicalPlan::from_logical(*input)),
                    expressions: exprs,
                    aliases,
                }
            }
            
            LogicalPlan::Sort { input, order_by } => {
                PhysicalPlan::Sort {
                    input: Box::new(PhysicalPlan::from_logical(*input)),
                    order_by,
                }
            }
            
            LogicalPlan::Limit { input, limit, offset } => {
                PhysicalPlan::Limit {
                    input: Box::new(PhysicalPlan::from_logical(*input)),
                    limit,
                    offset: offset.unwrap_or(0),
                }
            }
            
            LogicalPlan::Insert { table, columns, values } => {
                PhysicalPlan::Insert {
                    table,
                    columns,
                    values,
                }
            }
            
            LogicalPlan::Update { table, assignments, filter } => {
                PhysicalPlan::Update {
                    table,
                    assignments,
                    filter,
                }
            }
            
            LogicalPlan::Delete { table, filter } => {
                PhysicalPlan::Delete {
                    table,
                    filter,
                }
            }
            
            LogicalPlan::CreateTable { .. } => {
                // CREATE TABLE doesn't have a physical plan
                // It's handled directly by the engine
                panic!("CREATE TABLE should be handled by engine, not executor")
            }
        }
    }
}

impl std::fmt::Display for PhysicalPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhysicalPlan::TableScan { table } => {
                write!(f, "TableScan({})", table)
            }
            PhysicalPlan::IndexScan { table, index } => {
                write!(f, "IndexScan({}.{})", table, index)
            }
            PhysicalPlan::Filter { input, .. } => {
                write!(f, "Filter\n  {}", input)
            }
            PhysicalPlan::Project { input, expressions, .. } => {
                write!(f, "Project({} columns)\n  {}", expressions.len(), input)
            }
            PhysicalPlan::Sort { input, order_by } => {
                write!(f, "Sort({} keys)\n  {}", order_by.len(), input)
            }
            PhysicalPlan::Limit { input, limit, offset } => {
                if *offset > 0 {
                    write!(f, "Limit({} OFFSET {})\n  {}", limit, offset, input)
                } else {
                    write!(f, "Limit({})\n  {}", limit, input)
                }
            }
            PhysicalPlan::Insert { table, .. } => {
                write!(f, "Insert({})", table)
            }
            PhysicalPlan::Update { table, .. } => {
                write!(f, "Update({})", table)
            }
            PhysicalPlan::Delete { table, .. } => {
                write!(f, "Delete({})", table)
            }
        }
    }
}

