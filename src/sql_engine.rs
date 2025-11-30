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
use crate::planner::logical::LogicalPlan;
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
            Statement::CreateIndex(idx) => self.execute_create_index(idx),
            Statement::Begin => self.execute_begin(),
            Statement::Commit => self.execute_commit(),
            Statement::Rollback => self.execute_rollback(),
        }
    }
    
    /// Execute SELECT statement
    fn execute_select(&mut self, select: crate::sql::ast::SelectStatement) -> Result<QueryResult> {
        // Step 1: Build logical plan from AST
        let statement = Statement::Select(select);
        let mut logical_plan = PlanBuilder::new().build(statement)?;
        
        // Step 1.5: Expand SELECT * wildcards to actual column names
        logical_plan = self.expand_wildcards(logical_plan)?;
        
        // Step 2: Optimize logical plan
        let optimized = self.optimizer.optimize(logical_plan);
        
        // Step 3: Convert to physical plan
        let physical_plan = self.logical_to_physical(optimized)?;
        
        // Step 4: Compile physical plan to VM opcodes
        let program = self.compile_to_vm(&physical_plan)?;
        
        // Step 5: Execute VM program with table schemas
        let mut executor = Executor::new();
        let table_schemas = self.get_table_schemas();
        executor.execute(&program, &mut self.pager, &table_schemas)
    }
    
    /// Expand SELECT * wildcards to actual column names
    fn expand_wildcards(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        use crate::sql::ast::Expr;
        use crate::planner::logical::ProjectionExpr;
        
        match plan {
            LogicalPlan::Projection { input, expressions } => {
                // Check if any expression is a wildcard (SELECT *)
                let has_wildcard = expressions.iter().any(|proj| {
                    matches!(&proj.expr, Expr::Column { name, .. } if name == "*")
                });
                
                if has_wildcard {
                    // Extract table name from the scan
                    let table_name = self.extract_table_name(&input)?;
                    
                    // Get table schema
                    let schema = self.catalog.get_table(&table_name)
                        .ok_or_else(|| Error::Internal(format!("Table '{}' not found in catalog", table_name)))?;
                    
                    // Expand wildcard to actual columns
                    let mut expanded_exprs = Vec::new();
                    for proj in expressions {
                        if matches!(&proj.expr, Expr::Column { name, .. } if name == "*") {
                            // Replace * with all columns from table
                            for column in &schema.columns {
                                expanded_exprs.push(ProjectionExpr {
                                    expr: Expr::Column {
                                        table: None,
                                        name: column.name.clone(),
                                    },
                                    alias: None,
                                });
                            }
                        } else {
                            // Keep non-wildcard expressions as-is
                            expanded_exprs.push(proj);
                        }
                    }
                    
                    Ok(LogicalPlan::Projection {
                        input: Box::new(self.expand_wildcards(*input)?),
                        expressions: expanded_exprs,
                    })
                } else {
                    // No wildcard, recurse on input
                    Ok(LogicalPlan::Projection {
                        input: Box::new(self.expand_wildcards(*input)?),
                        expressions,
                    })
                }
            }
            LogicalPlan::Filter { input, predicate } => {
                Ok(LogicalPlan::Filter {
                    input: Box::new(self.expand_wildcards(*input)?),
                    predicate,
                })
            }
            LogicalPlan::Sort { input, order_by } => {
                Ok(LogicalPlan::Sort {
                    input: Box::new(self.expand_wildcards(*input)?),
                    order_by,
                })
            }
            LogicalPlan::Limit { input, limit, offset } => {
                Ok(LogicalPlan::Limit {
                    input: Box::new(self.expand_wildcards(*input)?),
                    limit,
                    offset,
                })
            }
            // Base cases - no wildcard possible
            _ => Ok(plan),
        }
    }
    
    /// Extract table name from a logical plan (find Scan node)
    fn extract_table_name(&self, plan: &LogicalPlan) -> Result<String> {
        match plan {
            LogicalPlan::Scan { table, .. } => Ok(table.clone()),
            LogicalPlan::Filter { input, .. } => self.extract_table_name(input),
            LogicalPlan::Projection { input, .. } => self.extract_table_name(input),
            LogicalPlan::Sort { input, .. } => self.extract_table_name(input),
            LogicalPlan::Limit { input, .. } => self.extract_table_name(input),
            _ => Err(Error::Internal("Cannot extract table name from plan".to_string())),
        }
    }
    
    /// Execute INSERT statement with constraint validation and auto-increment
    fn execute_insert(&mut self, mut insert: crate::sql::ast::InsertStatement) -> Result<QueryResult> {
        use crate::sql::ast::{Expr, Literal};
        
        #[cfg(test)]
        eprintln!("DEBUG: execute_insert called for table '{}'", insert.table);
        
        // Step 1: Get table schema from catalog
        let table_name = insert.table.clone();
        let table_schema = self.catalog.get_table(&table_name)
            .ok_or_else(|| Error::Internal(format!("Table '{}' does not exist", table_name)))?
            .clone(); // Clone to avoid borrow issues
        
        #[cfg(test)]
        eprintln!("DEBUG: Table schema columns:");
        #[cfg(test)]
        for (idx, col) in table_schema.columns.iter().enumerate() {
            eprintln!("  [{}] {} - nullable={}, primary_key={}", 
                idx, col.name, col.nullable, col.primary_key);
        }
        
        // Step 2: Process each row for validation and auto-increment
        let mut processed_rows = Vec::new();
        let mut last_id = table_schema.last_insert_id;
        
        for row_values in &insert.values {
            // Determine column mapping
            let column_indices: Vec<usize> = if let Some(ref cols) = insert.columns {
                // Specific columns provided - map to indices
                cols.iter()
                    .map(|col_name| {
                        table_schema.columns.iter()
                            .position(|c| &c.name == col_name)
                            .ok_or_else(|| Error::Internal(format!("Column '{}' not found", col_name)))
                    })
                    .collect::<Result<Vec<_>>>()?
            } else {
                // All columns in order
                (0..table_schema.columns.len()).collect()
            };
            
            // Validate value count
            if column_indices.len() != row_values.len() {
                return Err(Error::Internal(format!(
                    "Column count mismatch: {} columns but {} values",
                    column_indices.len(), row_values.len()
                )));
            }
            
            // Build full row with default/auto-increment values
            let mut full_row = vec![Expr::Literal(Literal::Null); table_schema.columns.len()];
            
            for (col_idx, value) in column_indices.iter().zip(row_values.iter()) {
                full_row[*col_idx] = value.clone();
            }
            
            // Handle auto-increment for INTEGER PRIMARY KEY
            if let Some(pk_idx) = table_schema.primary_key {
                let pk_column = &table_schema.columns[pk_idx];
                
                // Check if PRIMARY KEY is INTEGER and value is NULL or not provided
                if matches!(pk_column.data_type, crate::catalog::schema::ColumnType::Integer) {
                    let is_null_or_missing = matches!(&full_row[pk_idx], Expr::Literal(Literal::Null));
                    
                    if is_null_or_missing {
                        // Auto-increment: generate next ID
                        last_id += 1;
                        full_row[pk_idx] = Expr::Literal(Literal::Integer(last_id));
                    }
                }
            }
            
            // Validate NOT NULL constraints
            for (idx, value_expr) in full_row.iter().enumerate() {
                let column = &table_schema.columns[idx];
                
                #[cfg(test)]
                eprintln!("DEBUG: Checking column '{}' - nullable={}, value={:?}", 
                    column.name, column.nullable, value_expr);
                
                // nullable = true means NULL is allowed, so !nullable means NOT NULL
                if !column.nullable {
                    // Check if the value is NULL
                    let is_null = match value_expr {
                        Expr::Literal(Literal::Null) => true,
                        _ => false,
                    };
                    
                    #[cfg(test)]
                    eprintln!("DEBUG: Column '{}' is NOT NULL, value is_null={}", column.name, is_null);
                    
                    if is_null {
                        return Err(Error::Internal(format!(
                            "NOT NULL constraint violated for column '{}'. Column is marked as NOT NULL (nullable={})",
                            column.name, column.nullable
                        )));
                    }
                }
            }
            
            processed_rows.push(full_row);
        }
        
        // Update last_insert_id in catalog if it changed
        if last_id != table_schema.last_insert_id {
            // Update the schema in the catalog
            let mut updated_schema = table_schema.clone();
            updated_schema.last_insert_id = last_id;
            self.catalog.update_table(updated_schema)?;
            // Save catalog to persist the change
            self.catalog.save(&mut self.pager)?;
        }
        
        // Replace original values with processed rows
        insert.values = processed_rows;
        
        // Step 3: Convert to logical plan
        let statement = Statement::Insert(insert);
        let logical_plan = PlanBuilder::new().build(statement)?;
        
        // Step 4: Convert to physical plan
        let physical_plan = self.logical_to_physical(logical_plan)?;
        
        // Step 5: Compile to VM opcodes
        let program = self.compile_to_vm(&physical_plan)?;
        
        // Step 6: Execute with table schemas
        let mut executor = Executor::new();
        let table_schemas = self.get_table_schemas();
        let result = executor.execute(&program, &mut self.pager, &table_schemas)?;
        
        // Step 7: Validate UNIQUE constraints after insert
        self.validate_unique_constraints(&table_name, &table_schema)?;
        
        Ok(QueryResult::with_affected(result.rows_affected))
    }
    
    /// Validate UNIQUE constraints for a table
    fn validate_unique_constraints(&mut self, table_name: &str, table_schema: &crate::catalog::schema::TableSchema) -> Result<()> {
        use crate::storage::btree::BTree;
        use std::collections::HashSet;
        
        // For each column with UNIQUE constraint
        for (col_idx, column) in table_schema.columns.iter().enumerate() {
            if !column.unique && table_schema.primary_key != Some(col_idx) {
                continue; // Skip non-unique columns
            }
            
            // Scan table and collect values for this column
            let mut values_seen = HashSet::new();
            let btree = BTree::open(table_schema.root_page)?;
            let mut cursor = crate::storage::btree::Cursor::new(&mut self.pager, btree.root_page_id())?;
            
            loop {
                match cursor.current(&mut self.pager) {
                    Ok(record) => {
                        if col_idx < record.values.len() {
                            // Convert to string for comparison (simple approach)
                            let value_str = format!("{:?}", record.values[col_idx]);
                            
                            if !values_seen.insert(value_str.clone()) {
                                // Duplicate found!
                                return Err(Error::Internal(format!(
                                    "UNIQUE constraint violated for column '{}': duplicate value",
                                    column.name
                                )));
                            }
                        }
                        
                        // Move to next record
                        if cursor.next(&mut self.pager).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        
        Ok(())
    }
    
    /// Get table schemas as a HashMap for executor
    fn get_table_schemas(&self) -> std::collections::HashMap<String, crate::catalog::schema::TableSchema> {
        let mut schemas = std::collections::HashMap::new();
        for table_name in self.catalog.list_tables() {
            if let Some(schema) = self.catalog.get_table(&table_name) {
                schemas.insert(table_name, schema.clone());
            }
        }
        schemas
    }
    
    /// Execute UPDATE statement
    fn execute_update(&mut self, update: crate::sql::ast::UpdateStatement) -> Result<QueryResult> {
        // Step 1: Build logical plan
        let statement = Statement::Update(update);
        let logical_plan = PlanBuilder::new().build(statement)?;
        
        // Step 2: Optimize
        let optimized = self.optimizer.optimize(logical_plan);
        
        // Step 3: Convert to physical plan
        let physical_plan = self.logical_to_physical(optimized)?;
        
        // Step 4: Compile to VM opcodes
        let program = self.compile_to_vm(&physical_plan)?;
        
        // Step 5: Execute
        let mut executor = Executor::new();
        let table_schemas = self.get_table_schemas();
        let result = executor.execute(&program, &mut self.pager, &table_schemas)?;
        
        Ok(QueryResult::with_affected(result.rows_affected))
    }
    
    /// Execute DELETE statement
    fn execute_delete(&mut self, delete: crate::sql::ast::DeleteStatement) -> Result<QueryResult> {
        // Step 1: Build logical plan
        let statement = Statement::Delete(delete);
        let logical_plan = PlanBuilder::new().build(statement)?;
        
        // Step 2: Optimize
        let optimized = self.optimizer.optimize(logical_plan);
        
        // Step 3: Convert to physical plan
        let physical_plan = self.logical_to_physical(optimized)?;
        
        // Step 4: Compile to VM opcodes
        let program = self.compile_to_vm(&physical_plan)?;
        
        // Step 5: Execute
        let mut executor = Executor::new();
        let table_schemas = self.get_table_schemas();
        let result = executor.execute(&program, &mut self.pager, &table_schemas)?;
        
        Ok(QueryResult::with_affected(result.rows_affected))
    }
    
    /// Execute CREATE INDEX statement
    fn execute_create_index(&mut self, idx: crate::sql::ast::CreateIndexStatement) -> Result<QueryResult> {
        // Create index in catalog
        self.catalog.create_index(
            idx.name.clone(),
            idx.table.clone(),
            idx.columns.clone(),
            idx.unique,
            &mut self.pager,
        )?;
        
        // Save catalog
        self.catalog.save(&mut self.pager)?;
        
        eprintln!("✅ Index '{}' created on table '{}' for columns {:?}", 
                  idx.name, idx.table, idx.columns);
        
        Ok(QueryResult::with_affected(0))
    }
    
    /// Execute BEGIN statement
    fn execute_begin(&mut self) -> Result<QueryResult> {
        // TODO: Full implementation
        Ok(QueryResult::with_affected(0))
    }
    
    /// Execute COMMIT statement
    fn execute_commit(&mut self) -> Result<QueryResult> {
        // TODO: Full implementation
        Ok(QueryResult::with_affected(0))
    }
    
    /// Execute ROLLBACK statement
    fn execute_rollback(&mut self) -> Result<QueryResult> {
        // TODO: Full implementation
        Ok(QueryResult::with_affected(0))
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
            LP::CreateIndex { .. } | LP::Transaction { .. } => {
                // These are handled directly, not via physical plan
                Ok(PP::TableScan { table: "dummy".into() })
            }
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
        
        // Pass table schemas to compiler for column resolution
        let table_schemas = self.get_table_schemas();
        compiler.set_table_schemas(table_schemas);
        
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

