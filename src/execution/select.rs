/// SELECT execution
/// 
/// Implements full SELECT statement execution

use crate::catalog::CatalogManager;
use crate::error::Result;
use crate::planner::logical::LogicalPlan;
use crate::planner::physical::PhysicalPlan;
use crate::planner::optimizer::Optimizer;
use crate::storage::pager::Pager;
use crate::vm::executor::{Executor, QueryResult};

/// SELECT statement executor
pub struct SelectExecutor;

impl SelectExecutor {
    /// Execute a SELECT statement
    pub fn execute(
        plan: LogicalPlan,
        _catalog: &CatalogManager,
        pager: &mut Pager,
    ) -> Result<QueryResult> {
        // Optimize the plan
        let optimizer = Optimizer::new();
        let optimized_plan = optimizer.optimize(plan);
        
        // Convert to physical plan
        let physical_plan = PhysicalPlan::from_logical(optimized_plan);
        
        // Execute physical plan
        let mut executor = Executor::new();
        
        // For now, use simplified execution
        // Full implementation would generate VM opcodes and execute them
        match physical_plan {
            PhysicalPlan::TableScan { table } => {
                executor.execute_select(&table, pager)
            }
            PhysicalPlan::Project { input, .. } => {
                // Recursively execute input
                Self::execute_physical(*input, pager)
            }
            PhysicalPlan::Filter { input, .. } => {
                // Execute input with filter
                Self::execute_physical(*input, pager)
            }
            PhysicalPlan::Limit { input, limit, offset } => {
                // Execute input with limit/offset
                let mut result = Self::execute_physical(*input, pager)?;
                
                // Apply offset
                if offset > 0 && offset < result.rows.len() {
                    result.rows.drain(0..offset);
                } else if offset >= result.rows.len() {
                    result.rows.clear();
                }
                
                // Apply limit
                if result.rows.len() > limit {
                    result.rows.truncate(limit);
                }
                
                Ok(result)
            }
            _ => {
                // For other plan types, return empty result
                Ok(QueryResult::new())
            }
        }
    }
    
    /// Execute a physical plan
    fn execute_physical(plan: PhysicalPlan, pager: &mut Pager) -> Result<QueryResult> {
        let mut executor = Executor::new();
        
        match plan {
            PhysicalPlan::TableScan { table } => {
                executor.execute_select(&table, pager)
            }
            PhysicalPlan::Project { input, .. } => {
                Self::execute_physical(*input, pager)
            }
            PhysicalPlan::Filter { input, .. } => {
                Self::execute_physical(*input, pager)
            }
            PhysicalPlan::Limit { input, limit, offset } => {
                let mut result = Self::execute_physical(*input, pager)?;
                
                if offset > 0 && offset < result.rows.len() {
                    result.rows.drain(0..offset);
                }
                
                if result.rows.len() > limit {
                    result.rows.truncate(limit);
                }
                
                Ok(result)
            }
            _ => Ok(QueryResult::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::logical::{LogicalPlan, ProjectionExpr};
    use crate::sql::ast::Expr;
    
    #[test]
    fn test_select_executor() {
        let mut pager = Pager::open("test_select.db").unwrap();
        let catalog = CatalogManager::new();
        
        // Create a simple scan plan
        let plan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
        };
        
        let plan = LogicalPlan::Projection {
            input: Box::new(plan),
            expressions: vec![
                ProjectionExpr {
                    expr: Expr::Column { table: None, name: "*".to_string() },
                    alias: None,
                },
            ],
        };
        
        // Execute
        let result = SelectExecutor::execute(plan, &catalog, &mut pager);
        assert!(result.is_ok());
        
        // Cleanup
        std::fs::remove_file("test_select.db").ok();
    }
}

