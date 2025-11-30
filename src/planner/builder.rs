/// Plan Builder - Converts AST to Logical Plan
/// 
/// Builds logical query plans from SQL AST

use crate::error::{Error, Result};
use crate::planner::logical::*;
use crate::sql::ast::*;

/// Builds logical plans from SQL AST
pub struct PlanBuilder;

impl PlanBuilder {
    /// Create a new plan builder
    pub fn new() -> Self {
        PlanBuilder
    }
    
    /// Build a logical plan from a SQL statement
    pub fn build(&self, statement: Statement) -> Result<LogicalPlan> {
        match statement {
            Statement::Select(select) => self.build_select(select),
            Statement::Insert(insert) => self.build_insert(insert),
            Statement::Update(update) => self.build_update(update),
            Statement::Delete(delete) => self.build_delete(delete),
            Statement::CreateTable(create) => self.build_create_table(create),
            Statement::CreateIndex(idx) => self.build_create_index(idx),
            Statement::Begin => Ok(LogicalPlan::Transaction { operation: "BEGIN".to_string() }),
            Statement::Commit => Ok(LogicalPlan::Transaction { operation: "COMMIT".to_string() }),
            Statement::Rollback => Ok(LogicalPlan::Transaction { operation: "ROLLBACK".to_string() }),
        }
    }
    
    /// Build SELECT plan
    fn build_select(&self, select: SelectStatement) -> Result<LogicalPlan> {
        // Start with table scan
        let table = select.from.ok_or_else(|| {
            Error::Internal("SELECT must have FROM clause".to_string())
        })?;
        
        let mut plan = LogicalPlan::Scan {
            table,
            alias: None,
        };
        
        // Add WHERE filter
        if let Some(predicate) = select.where_clause {
            plan = LogicalPlan::Filter {
                input: Box::new(plan),
                predicate,
            };
        }
        
        // Add projection
        let expressions = select.columns.into_iter().map(|col| {
            match col {
                SelectColumn::Star => ProjectionExpr {
                    expr: Expr::Column { table: None, name: "*".to_string() },
                    alias: None,
                },
                SelectColumn::Expr { expr, alias } => ProjectionExpr {
                    expr,
                    alias,
                },
            }
        }).collect();
        
        plan = LogicalPlan::Projection {
            input: Box::new(plan),
            expressions,
        };
        
        // Add ORDER BY
        if !select.order_by.is_empty() {
            plan = LogicalPlan::Sort {
                input: Box::new(plan),
                order_by: select.order_by,
            };
        }
        
        // Add LIMIT/OFFSET
        if let Some(limit) = select.limit {
            plan = LogicalPlan::Limit {
                input: Box::new(plan),
                limit,
                offset: select.offset,
            };
        }
        
        Ok(plan)
    }
    
    /// Build INSERT plan
    fn build_insert(&self, insert: InsertStatement) -> Result<LogicalPlan> {
        Ok(LogicalPlan::Insert {
            table: insert.table,
            columns: insert.columns,
            values: insert.values,
        })
    }
    
    /// Build UPDATE plan
    fn build_update(&self, update: UpdateStatement) -> Result<LogicalPlan> {
        let assignments = update.assignments.into_iter()
            .map(|a| (a.column, a.value))
            .collect();
        
        Ok(LogicalPlan::Update {
            table: update.table,
            assignments,
            filter: update.where_clause,
        })
    }
    
    /// Build DELETE plan
    fn build_delete(&self, delete: DeleteStatement) -> Result<LogicalPlan> {
        Ok(LogicalPlan::Delete {
            table: delete.table,
            filter: delete.where_clause,
        })
    }
    
    /// Build CREATE INDEX plan
    fn build_create_index(&self, idx: CreateIndexStatement) -> Result<LogicalPlan> {
        Ok(LogicalPlan::CreateIndex {
            name: idx.name,
            table: idx.table,
            columns: idx.columns,
            unique: idx.unique,
        })
    }
    
    /// Build CREATE TABLE plan
    fn build_create_table(&self, create: CreateTableStatement) -> Result<LogicalPlan> {
        let columns = create.columns.into_iter().map(|col| {
            let data_type = match col.data_type {
                crate::sql::ast::DataType::Integer => crate::planner::logical::DataType::Integer,
                crate::sql::ast::DataType::Real => crate::planner::logical::DataType::Real,
                crate::sql::ast::DataType::Text => crate::planner::logical::DataType::Text,
                crate::sql::ast::DataType::Blob => crate::planner::logical::DataType::Blob,
            };
            
            let mut not_null = false;
            let mut primary_key = false;
            let mut unique = false;
            
            for constraint in &col.constraints {
                match constraint {
                    crate::sql::ast::ColumnConstraint::NotNull => not_null = true,
                    crate::sql::ast::ColumnConstraint::PrimaryKey => {
                        primary_key = true;
                        not_null = true; // PRIMARY KEY implies NOT NULL
                    }
                    crate::sql::ast::ColumnConstraint::Unique => unique = true,
                    crate::sql::ast::ColumnConstraint::Default(_) => {
                        // TODO: Parse default value
                    }
                }
            }
            
            ColumnSpec {
                name: col.name,
                data_type,
                not_null,
                primary_key,
                unique,
                default: None,
            }
        }).collect();
        
        Ok(LogicalPlan::CreateTable {
            table: create.table,
            columns,
        })
    }
}

impl Default for PlanBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::{Lexer, Parser};
    
    #[test]
    fn test_build_select_plan() {
        let sql = "SELECT name, age FROM users WHERE age > 18";
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let stmt = parser.parse_statement().unwrap();
        
        let builder = PlanBuilder::new();
        let plan = builder.build(stmt).unwrap();
        
        // Should have Projection -> Filter -> Scan structure
        assert!(matches!(plan, LogicalPlan::Projection { .. }));
    }
    
    #[test]
    fn test_build_insert_plan() {
        let sql = "INSERT INTO users (name, age) VALUES ('Alice', 30)";
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let stmt = parser.parse_statement().unwrap();
        
        let builder = PlanBuilder::new();
        let plan = builder.build(stmt).unwrap();
        
        assert!(matches!(plan, LogicalPlan::Insert { .. }));
    }
}

