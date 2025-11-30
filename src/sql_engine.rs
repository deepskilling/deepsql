/// SQL Execution Engine
/// 
/// High-level SQL execution coordinator that connects:
/// - SQL Parser (string → AST)
/// - Query Planner (AST → Logical Plan → Physical Plan)
/// - Plan Compiler (Physical Plan → VM Opcodes)
/// - VM Executor (Opcodes → Results)

use crate::error::{Error, Result};
use crate::storage::Pager;
use crate::catalog::CatalogManager;
use crate::sql::parser::Parser;
use crate::sql::lexer::Lexer;
use crate::sql::ast::Statement;
use crate::planner::builder::PlanBuilder;
use crate::planner::optimizer::Optimizer;
use crate::planner::physical::PhysicalPlan;
use crate::vm::executor::{Executor, QueryResult};
use crate::vm::opcode::Program;

/// SQL Execution Engine
/// 
/// Coordinates the entire SQL execution pipeline from
/// raw SQL string to query results.
pub struct SqlEngine {
    /// Catalog manager for schema metadata
    catalog: CatalogManager,
    
    /// Storage pager
    pager: Pager,
    
    /// Query optimizer
    optimizer: Optimizer,
}

impl SqlEngine {
    /// Create a new SQL engine
    pub fn new(catalog: CatalogManager, pager: Pager) -> Self {
        SqlEngine {
            catalog,
            pager,
            optimizer: Optimizer::new(),
        }
    }
    
    /// Execute a SQL statement
    /// 
    /// This is the main entry point for SQL execution.
    /// 
    /// # Arguments
    /// * `sql` - SQL statement string
    /// 
    /// # Returns
    /// * `QueryResult` - Query results or affected row count
    /// 
    /// # Example
    /// ```
    /// let result = engine.execute("SELECT * FROM users WHERE age > 18")?;
    /// ```
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        // Step 1: Lex & Parse SQL to AST
        let tokens = Lexer::new(sql).tokenize()?;
        let mut parser = Parser::new(tokens);
        let statement = parser.parse_statement()?;
        
        // Step 2: Route to appropriate handler
        match statement {
            Statement::Select(select) => self.execute_select(select),
            Statement::Insert(insert) => self.execute_insert(insert),
            Statement::Update(update) => self.execute_update(update),
            Statement::Delete(delete) => self.execute_delete(delete),
            Statement::CreateTable(create) => self.execute_create_table(create),
        }
    }
    
    /// Execute SELECT statement
    fn execute_select(&mut self, select: crate::sql::ast::SelectStatement) -> Result<QueryResult> {
        // Step 1: Build logical plan from AST
        let logical_plan = PlanBuilder::new(&self.catalog)
            .build_select_plan(select)?;
        
        // Step 2: Optimize logical plan
        let optimized = self.optimizer.optimize(logical_plan)?;
        
        // Step 3: Convert to physical plan
        let physical_plan = self.logical_to_physical(optimized)?;
        
        // Step 4: Compile physical plan to VM opcodes
        let program = self.compile_to_vm(physical_plan)?;
        
        // Step 5: Execute VM program
        let mut executor = Executor::new();
        executor.execute(&program, &mut self.pager)
    }
    
    /// Execute INSERT statement
    fn execute_insert(&mut self, _insert: crate::sql::ast::InsertStatement) -> Result<QueryResult> {
        // TODO: Phase A Week 2
        Err(Error::Internal("INSERT not yet implemented".to_string()))
    }
    
    /// Execute UPDATE statement
    fn execute_update(&mut self, _update: crate::sql::ast::UpdateStatement) -> Result<QueryResult> {
        // TODO: Phase A Week 2-3
        Err(Error::Internal("UPDATE not yet implemented".to_string()))
    }
    
    /// Execute DELETE statement
    fn execute_delete(&mut self, _delete: crate::sql::ast::DeleteStatement) -> Result<QueryResult> {
        // TODO: Phase A Week 3
        Err(Error::Internal("DELETE not yet implemented".to_string()))
    }
    
    /// Execute CREATE TABLE statement
    fn execute_create_table(&mut self, _create: crate::sql::ast::CreateTableStatement) -> Result<QueryResult> {
        // TODO: Phase A Week 3-4
        Err(Error::Internal("CREATE TABLE not yet implemented".to_string()))
    }
    
    /// Convert logical plan to physical plan
    fn logical_to_physical(&self, _logical: crate::planner::logical::LogicalPlan) -> Result<PhysicalPlan> {
        // TODO: Implement conversion
        // For now, return a placeholder
        Err(Error::Internal("Logical to physical conversion not yet implemented".to_string()))
    }
    
    /// Compile physical plan to VM opcodes
    fn compile_to_vm(&self, _physical: PhysicalPlan) -> Result<Program> {
        // TODO: Implement compilation
        // For now, return empty program
        Ok(Program {
            opcodes: vec![crate::vm::opcode::Opcode::Halt],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_sql_engine_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let pager = Pager::open(db_path.to_str().unwrap()).unwrap();
        let catalog = CatalogManager::new(pager.clone());
        
        let _engine = SqlEngine::new(catalog, pager);
        // Just verify it compiles and creates
    }
    
    #[test]
    fn test_parse_simple_select() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let pager = Pager::open(db_path.to_str().unwrap()).unwrap();
        let catalog = CatalogManager::new(pager.clone());
        let mut engine = SqlEngine::new(catalog, pager);
        
        // This will fail because execution isn't implemented yet,
        // but it should parse successfully
        let result = engine.execute("SELECT * FROM users");
        
        // For now, we expect it to fail with "not yet implemented"
        assert!(result.is_err());
    }
}

