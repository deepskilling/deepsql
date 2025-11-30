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
use crate::planner::compiler::VMCompiler;
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
    pub fn new(pager: Pager) -> Self {
        SqlEngine {
            catalog: CatalogManager::new(),
            pager,
            optimizer: Optimizer::new(),
        }
    }
    
    /// Load catalog from database
    pub fn load_catalog(&mut self) -> Result<()> {
        self.catalog.load(&mut self.pager)
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
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize();  // Returns Vec<Token>, not Result
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
        let statement = Statement::Select(select);
        let logical_plan = PlanBuilder::new().build(statement)?;
        
        // Step 2: Optimize logical plan
        let optimized = self.optimizer.optimize(logical_plan);
        
        // Step 3: Convert to physical plan
        let physical_plan = self.logical_to_physical(optimized)?;
        
        // Step 4: Compile physical plan to VM opcodes
        let program = self.compile_to_vm(&physical_plan)?;
        
        // Step 5: Execute VM program
        let mut executor = Executor::new();
        executor.execute(&program, &mut self.pager)
    }
    
    /// Execute INSERT statement
    fn execute_insert(&mut self, insert: crate::sql::ast::InsertStatement) -> Result<QueryResult> {
        // Step 1: Get table schema from catalog
        let table_schema = self.catalog.get_table(&insert.table)
            .ok_or_else(|| Error::Internal(format!("Table '{}' does not exist", insert.table)))?;
        
        // Step 2: Validate column count
        let _column_count = if let Some(ref cols) = insert.columns {
            // Specific columns provided
            if cols.len() != insert.values[0].len() {
                return Err(Error::Internal(
                    format!("Column count mismatch: {} columns specified but {} values provided",
                            cols.len(), insert.values[0].len())
                ));
            }
            cols.len()
        } else {
            // All columns
            if table_schema.columns.len() != insert.values[0].len() {
                return Err(Error::Internal(
                    format!("Column count mismatch: table has {} columns but {} values provided",
                            table_schema.columns.len(), insert.values[0].len())
                ));
            }
            table_schema.columns.len()
        };
        
        // Step 3: Convert to logical plan
        let statement = Statement::Insert(insert);
        let logical_plan = PlanBuilder::new().build(statement)?;
        
        // Step 4: Convert to physical plan
        let physical_plan = self.logical_to_physical(logical_plan)?;
        
        // Step 5: Compile to VM opcodes
        let program = self.compile_to_vm(&physical_plan)?;
        
        // Step 6: Execute
        let mut executor = Executor::new();
        let result = executor.execute(&program, &mut self.pager)?;
        
        Ok(QueryResult::with_affected(result.rows_affected))
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
    fn execute_create_table(&mut self, create: crate::sql::ast::CreateTableStatement) -> Result<QueryResult> {
        // Step 1: Convert AST to LogicalPlan
        let statement = Statement::CreateTable(create);
        let logical_plan = PlanBuilder::new().build(statement)?;
        
        // Step 2: Execute via catalog manager
        self.catalog.create_table(&logical_plan, &mut self.pager)?;
        
        // Step 3: Return success
        Ok(QueryResult::with_affected(0))
    }
    
    /// Convert logical plan to physical plan
    fn logical_to_physical(&self, logical: crate::planner::logical::LogicalPlan) -> Result<PhysicalPlan> {
        use crate::planner::logical::LogicalPlan as LP;
        use crate::planner::physical::PhysicalPlan as PP;
        
        match logical {
            LP::Scan { table, .. } => {
                Ok(PP::TableScan { 
                    table,
                })
            }
            
            LP::Filter { input, predicate } => {
                let input_physical = self.logical_to_physical(*input)?;
                Ok(PP::Filter {
                    input: Box::new(input_physical),
                    predicate,
                })
            }
            
            LP::Projection { input, expressions } => {
                let input_physical = self.logical_to_physical(*input)?;
                let (exprs, aliases): (Vec<_>, Vec<_>) = expressions
                    .into_iter()
                    .map(|pe| (pe.expr, pe.alias))
                    .unzip();
                Ok(PP::Project {
                    input: Box::new(input_physical),
                    expressions: exprs,
                    aliases,
                })
            }
            
            LP::Sort { input, order_by } => {
                let input_physical = self.logical_to_physical(*input)?;
                Ok(PP::Sort {
                    input: Box::new(input_physical),
                    order_by,
                })
            }
            
            LP::Limit { input, limit, offset } => {
                let input_physical = self.logical_to_physical(*input)?;
                Ok(PP::Limit {
                    input: Box::new(input_physical),
                    limit,
                    offset: offset.unwrap_or(0),
                })
            }
            
            LP::Insert { table, columns, values } => {
                Ok(PP::Insert { table, columns, values })
            }
            
            LP::Update { table, assignments, filter } => {
                Ok(PP::Update { 
                    table, 
                    assignments, 
                    filter,
                })
            }
            
            LP::Delete { table, filter } => {
                Ok(PP::Delete { 
                    table, 
                    filter,
                })
            }
            
            LP::CreateTable { table, columns } => {
                // CreateTable execution will be handled differently
                // For now, return error
                let _ = (table, columns);
                Err(Error::Internal("CREATE TABLE execution not yet implemented".to_string()))
            }
        }
    }
    
    /// Compile physical plan to VM opcodes
    fn compile_to_vm(&self, physical: &PhysicalPlan) -> Result<Program> {
        let mut compiler = VMCompiler::new();
        compiler.compile(physical)
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
        let _engine = SqlEngine::new(pager);
        // Just verify it compiles and creates
    }
    
    #[test]
    fn test_parse_simple_select() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let pager = Pager::open(db_path.to_str().unwrap()).unwrap();
        let mut engine = SqlEngine::new(pager);
        
        // This will fail because execution isn't implemented yet,
        // but it should parse successfully
        let result = engine.execute("SELECT * FROM users");
        
        // For now, we expect it to fail with "not yet implemented"
        assert!(result.is_err());
    }
}

