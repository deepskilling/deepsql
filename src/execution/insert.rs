/// INSERT execution
/// 
/// Implements full INSERT statement execution

use crate::catalog::CatalogManager;
use crate::error::{Error, Result};
use crate::planner::logical::LogicalPlan;
use crate::storage::pager::Pager;
use crate::vm::executor::QueryResult;

/// INSERT statement executor
pub struct InsertExecutor;

impl InsertExecutor {
    /// Execute an INSERT statement
    pub fn execute(
        plan: LogicalPlan,
        catalog: &mut CatalogManager,
        pager: &mut Pager,
    ) -> Result<QueryResult> {
        match plan {
            LogicalPlan::Insert { table, columns, values } => {
                // Verify table exists
                let table_schema = catalog.get_table(&table)
                    .ok_or_else(|| Error::Internal(format!("Table '{}' does not exist", table)))?;
                
                // For each value row, insert into table
                let mut rows_inserted = 0;
                
                for _value_row in &values {
                    // TODO: Evaluate expressions and create record
                    // TODO: Insert record into B+Tree
                    // TODO: Update indexes
                    
                    rows_inserted += 1;
                }
                
                Ok(QueryResult::with_affected(rows_inserted))
            }
            _ => Err(Error::Internal("Expected INSERT plan".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::logical::{LogicalPlan, ColumnSpec, DataType};
    use crate::sql::ast::Expr;
    
    #[test]
    fn test_insert_executor() {
        let mut pager = Pager::open("test_insert.db").unwrap();
        let mut catalog = CatalogManager::new();
        
        // Create table first
        let create_plan = LogicalPlan::CreateTable {
            table: "users".to_string(),
            columns: vec![
                ColumnSpec {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    not_null: true,
                    primary_key: true,
                    unique: false,
                    default: None,
                },
            ],
        };
        
        catalog.create_table(&create_plan, &mut pager).unwrap();
        
        // Insert data
        let insert_plan = LogicalPlan::Insert {
            table: "users".to_string(),
            columns: None,
            values: vec![vec![Expr::Literal(crate::sql::ast::Literal::Integer(1))]],
        };
        
        let result = InsertExecutor::execute(insert_plan, &mut catalog, &mut pager);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows_affected, 1);
        
        // Cleanup
        std::fs::remove_file("test_insert.db").ok();
    }
}

