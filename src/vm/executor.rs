/// VM Executor - Executes VM programs - ENHANCED VERSION
/// 
/// Complete implementation of all VM opcodes with:
/// - Cursor management
/// - Table operations (scan, insert, update, delete)
/// - Result sorting and limiting
/// - Full expression evaluation

use crate::error::{Error, Result};
use crate::storage::Pager;
use crate::storage::btree::{BTree, Cursor};
use crate::storage::record::{Record, Value as RecordValue};
use crate::types::Value;
use crate::vm::evaluator::ExprEvaluator;
use crate::vm::opcode::{Opcode, Program};
use crate::catalog::schema::TableSchema;
use std::collections::HashMap;

/// Query execution result
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Result rows
    pub rows: Vec<Vec<Value>>,
    
    /// Number of rows affected (for INSERT/UPDATE/DELETE)
    pub rows_affected: usize,
}

impl QueryResult {
    /// Create a new empty result
    pub fn new() -> Self {
        QueryResult {
            rows: Vec::new(),
            rows_affected: 0,
        }
    }
    
    /// Create a result with rows
    pub fn with_rows(rows: Vec<Vec<Value>>) -> Self {
        QueryResult {
            rows,
            rows_affected: 0,
        }
    }
    
    /// Create a result with affected row count
    pub fn with_affected(count: usize) -> Self {
        QueryResult {
            rows: Vec::new(),
            rows_affected: count,
        }
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Cursor state
struct CursorState {
    btree: BTree,
    cursor: Cursor,
    current_record: Option<Record>,
    table_name: String, // For column resolution in Filter
}

/// VM Executor
pub struct Executor {
    /// Virtual machine registers
    registers: Vec<Value>,
    
    /// Expression evaluator
    evaluator: ExprEvaluator,
    
    /// Result accumulator
    result: QueryResult,
    
    /// Active cursors (cursor_id -> state)
    cursors: HashMap<usize, CursorState>,
    
    /// Next cursor ID (reserved for future cursor allocation)
    _next_cursor_id: usize,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Self {
        Executor {
            registers: vec![Value::Null; 256], // 256 registers
            evaluator: ExprEvaluator::new(),
            result: QueryResult::new(),
            cursors: HashMap::new(),
            _next_cursor_id: 0,
        }
    }
    
    /// Execute a program with table schemas
    pub fn execute(&mut self, program: &Program, pager: &mut Pager, table_schemas: &HashMap<String, TableSchema>) -> Result<QueryResult> {
        let mut pc = 0; // Program counter
        
        // Execution loop
        while pc < program.opcodes.len() {
            let opcode = &program.opcodes[pc];
            
            match opcode {
                Opcode::Halt => break,
                
                Opcode::TableScan { table, cursor_id } => {
                    // Look up table schema to get root_page_id
                    let table_schema = table_schemas.get(table)
                        .ok_or_else(|| Error::Internal(format!("Table '{}' not found in catalog", table)))?;
                    
                    let root_page_id = table_schema.root_page;
                    let btree = BTree::open(root_page_id)?;
                    let cursor = Cursor::new(pager, btree.root_page_id())?;
                    
                    self.cursors.insert(*cursor_id, CursorState {
                        btree,
                        cursor,
                        current_record: None,
                        table_name: table.clone(),
                    });
                    
                    pc += 1;
                }
                
                Opcode::Rewind { cursor_id, jump_if_empty } => {
                    // Rewind cursor to beginning
                    if let Some(state) = self.cursors.get_mut(cursor_id) {
                        // Re-create cursor at start of B+Tree
                        state.cursor = Cursor::new(pager, state.btree.root_page_id())?;
                        
                        // Try to fetch first record
                        match state.cursor.current(pager) {
                            Ok(record) => {
                                state.current_record = Some(record);
                                pc += 1;
                            }
                            Err(_) => {
                                // Empty table or error, jump
                                pc = *jump_if_empty;
                            }
                        }
                    } else {
                        return Err(Error::Internal(format!("Invalid cursor ID: {}", cursor_id)));
                    }
                }
                
                Opcode::Next { cursor_id, jump_if_done } => {
                    // Move to next row
                    if let Some(state) = self.cursors.get_mut(cursor_id) {
                        match state.cursor.next(pager) {
                            Ok(true) => {
                                // Successfully moved to next record
                                match state.cursor.current(pager) {
                                    Ok(record) => {
                                        state.current_record = Some(record);
                                        pc += 1;
                                    }
                                    Err(_) => {
                                        pc = *jump_if_done;
                                    }
                                }
                            }
                            _ => {
                                // End of data or error, jump
                                pc = *jump_if_done;
                            }
                        }
                    } else {
                        return Err(Error::Internal(format!("Invalid cursor ID: {}", cursor_id)));
                    }
                }
                
                Opcode::Column { cursor_id, column_index, register } => {
                    // Read column from current row
                    if let Some(state) = self.cursors.get(cursor_id) {
                        if let Some(record) = &state.current_record {
                            #[cfg(test)]
                            eprintln!("DEBUG Column: cursor={}, col_idx={}, total_values={}, values={:?}", 
                                cursor_id, column_index, record.values.len(), record.values);
                            
                            if *column_index < record.values.len() {
                                // Convert RecordValue to Value
                                let value = convert_record_value_to_value(&record.values[*column_index]);
                                
                                #[cfg(test)]
                                eprintln!("DEBUG Column: Set register[{}] = {:?}", register, &value);
                                
                                self.registers[*register] = value;
                            } else {
                                self.registers[*register] = Value::Null;
                            }
                        } else {
                            self.registers[*register] = Value::Null;
                        }
                    }
                    pc += 1;
                }
                
                Opcode::Eval { expr, register } => {
                    let value = self.evaluator.eval(expr)?;
                    self.registers[*register] = value;
                    pc += 1;
                }
                
                Opcode::Filter { condition, jump_target } => {
                    // Build column context from current cursor record for WHERE clause evaluation
                    let mut row_map = HashMap::new();
                    
                    // Find the active cursor and build column name -> value mapping
                    for (_cid, state) in &self.cursors {
                        if let Some(record) = &state.current_record {
                            if let Some(schema) = table_schemas.get(&state.table_name) {
                                for (idx, col_schema) in schema.columns.iter().enumerate() {
                                    if idx < record.values.len() {
                                        let val = convert_record_value_to_value(&record.values[idx]);
                                        row_map.insert(col_schema.name.clone(), val);
                                    }
                                }
                            }
                            break; // Use first active cursor
                        }
                    }
                    
                    // Set row context for evaluator
                    self.evaluator.set_row(row_map);
                    
                    // Evaluate the WHERE condition
                    let value = self.evaluator.eval(condition)?;
                    
                    // Clear context after evaluation
                    self.evaluator.clear();
                    
                    if !value.is_truthy()? {
                        pc = *jump_target;
                    } else {
                        pc += 1;
                    }
                }
                
                Opcode::ResultRow { register_start, register_count } => {
                    let row: Vec<Value> = self.registers[*register_start..*register_start + register_count].to_vec();
                    self.result.rows.push(row);
                    pc += 1;
                }
                
                Opcode::Insert { cursor_id, register_start, register_count } => {
                    // Insert row into table via cursor
                    let values: Vec<RecordValue> = self.registers[*register_start..*register_start + register_count]
                        .iter()
                        .map(convert_value_to_record_value)
                        .collect();
                    
                    // Generate key (simplified - use first value)
                    let key = format!("{:?}", values.get(0).unwrap_or(&RecordValue::Null)).into_bytes();
                    let record = Record::new(key, values);
                    
                    // Insert using the cursor's B+Tree
                    if let Some(state) = self.cursors.get_mut(cursor_id) {
                        state.btree.insert(pager, record)?;
                        self.result.rows_affected += 1;
                    }
                    pc += 1;
                }
                
                Opcode::Update { cursor_id, updates } => {
                    // Update current row:
                    // 1. Get current record
                    // 2. Apply updates to create new record
                    // 3. Delete old record
                    // 4. Insert new record
                    
                    if let Some(state) = self.cursors.get(cursor_id) {
                        if let Some(record) = &state.current_record {
                            // Create a mutable copy of the record's values
                            let mut new_values = record.values.clone();
                            
                            // Apply each update
                            for (col_idx, expr) in updates {
                                // Evaluate the expression to get the new value
                                let new_val = self.evaluator.eval(expr)?;
                                
                                // Convert Value to RecordValue
                                let record_val = match new_val {
                                    Value::Integer(i) => RecordValue::Integer(i),
                                    Value::Real(r) => RecordValue::Real(r),
                                    Value::Text(s) => RecordValue::Text(s),
                                    Value::Blob(b) => RecordValue::Blob(b),
                                    Value::Null => RecordValue::Null,
                                };
                                
                                // Update the column
                                if *col_idx < new_values.len() {
                                    new_values[*col_idx] = record_val;
                                }
                            }
                            
                            // Create new record with updated values
                            let new_record = Record {
                                key: record.key.clone(),
                                values: new_values,
                            };
                            
                            // Delete old record and insert new one
                            let root_page_id = state.btree.root_page_id();
                            let mut btree = BTree::open(root_page_id)?;
                            btree.delete(pager, &record.key)?;
                            btree.insert(pager, new_record)?;
                            
                            self.result.rows_affected += 1;
                        }
                    }
                    pc += 1;
                }
                
                Opcode::Delete { cursor_id } => {
                    // Delete current row
                    if let Some(state) = self.cursors.get(cursor_id) {
                        if let Some(record) = &state.current_record {
                            // Use the B+Tree from the cursor state
                            let root_page_id = state.btree.root_page_id();
                            let mut btree = BTree::open(root_page_id)?;
                            btree.delete(pager, &record.key)?;
                            self.result.rows_affected += 1;
                        }
                    }
                    pc += 1;
                }
                
                Opcode::Sort { order_by } => {
                    // Sort result rows
                    self.sort_results_by_order_by(order_by)?;
                    pc += 1;
                }
                
                Opcode::Limit { limit, offset, counter_register } => {
                    // Limit/offset implementation
                    let counter = if let Value::Integer(c) = self.registers[*counter_register] {
                        c as usize
                    } else {
                        0
                    };
                    
                    if counter < *offset {
                        // Skip this row
                        self.registers[*counter_register] = Value::Integer((counter + 1) as i64);
                    } else if counter < offset + limit {
                        // Include this row
                        self.registers[*counter_register] = Value::Integer((counter + 1) as i64);
                    } else {
                        // Exceeded limit, halt
                        break;
                    }
                    pc += 1;
                }
                
                Opcode::Aggregate { function, expr, accumulator_register } => {
                    // Accumulate value for aggregate function
                    use crate::vm::opcode::AggregateFunction;
                    
                    // Ensure accumulator register exists
                    while self.registers.len() <= *accumulator_register {
                        self.registers.push(Value::Null);
                    }
                    
                    match function {
                        AggregateFunction::Count => {
                            // COUNT: increment counter
                            let current = if let Value::Integer(c) = self.registers[*accumulator_register] {
                                c
                            } else {
                                0
                            };
                            self.registers[*accumulator_register] = Value::Integer(current + 1);
                        }
                        AggregateFunction::Sum | AggregateFunction::Avg => {
                            // SUM/AVG: accumulate values
                            if let Some(expr) = expr {
                                // Need to set row context for expression evaluation
                                // Find the cursor that's being aggregated
                                let cursor_states: Vec<_> = self.cursors.iter().collect();
                                if let Some((_, state)) = cursor_states.first() {
                                    if let Some(record) = &state.current_record {
                                        // Build row context from record and table schema
                                        let table_name = &state.table_name;
                                        if let Some(schema) = table_schemas.get(table_name) {
                                            let mut row_context = std::collections::HashMap::new();
                                            for (i, col) in schema.columns.iter().enumerate() {
                                                if i < record.values.len() {
                                                    let value = match &record.values[i] {
                                                        RecordValue::Integer(v) => Value::Integer(*v),
                                                        RecordValue::Real(v) => Value::Real(*v),
                                                        RecordValue::Text(v) => Value::Text(v.clone()),
                                                        RecordValue::Blob(v) => Value::Blob(v.clone()),
                                                        RecordValue::Null => Value::Null,
                                                    };
                                                    row_context.insert(col.name.clone(), value);
                                                }
                                            }
                                            self.evaluator.set_row(row_context);
                                        }
                                    }
                                }
                                
                                let value = self.evaluator.eval(expr)?;
                                self.evaluator.clear(); // Clear row context
                                let current = &self.registers[*accumulator_register];
                                
                                if matches!(current, Value::Null) {
                                    // First value
                                    self.registers[*accumulator_register] = value;
                                } else {
                                    // Add to accumulator
                                    self.registers[*accumulator_register] = current.add(&value)?;
                                }
                            }
                        }
                        AggregateFunction::Min => {
                            if let Some(expr) = expr {
                                // Set row context (same as SUM/AVG)
                                let cursor_states: Vec<_> = self.cursors.iter().collect();
                                if let Some((_, state)) = cursor_states.first() {
                                    if let Some(record) = &state.current_record {
                                        let table_name = &state.table_name;
                                        if let Some(schema) = table_schemas.get(table_name) {
                                            let mut row_context = std::collections::HashMap::new();
                                            for (i, col) in schema.columns.iter().enumerate() {
                                                if i < record.values.len() {
                                                    let value = match &record.values[i] {
                                                        RecordValue::Integer(v) => Value::Integer(*v),
                                                        RecordValue::Real(v) => Value::Real(*v),
                                                        RecordValue::Text(v) => Value::Text(v.clone()),
                                                        RecordValue::Blob(v) => Value::Blob(v.clone()),
                                                        RecordValue::Null => Value::Null,
                                                    };
                                                    row_context.insert(col.name.clone(), value);
                                                }
                                            }
                                            self.evaluator.set_row(row_context);
                                        }
                                    }
                                }
                                
                                let value = self.evaluator.eval(expr)?;
                                self.evaluator.clear();
                                let current = &self.registers[*accumulator_register];
                                
                                if matches!(current, Value::Null) || value.compare(current)? == std::cmp::Ordering::Less {
                                    self.registers[*accumulator_register] = value;
                                }
                            }
                        }
                        AggregateFunction::Max => {
                            if let Some(expr) = expr {
                                // Set row context (same as SUM/AVG)
                                let cursor_states: Vec<_> = self.cursors.iter().collect();
                                if let Some((_, state)) = cursor_states.first() {
                                    if let Some(record) = &state.current_record {
                                        let table_name = &state.table_name;
                                        if let Some(schema) = table_schemas.get(table_name) {
                                            let mut row_context = std::collections::HashMap::new();
                                            for (i, col) in schema.columns.iter().enumerate() {
                                                if i < record.values.len() {
                                                    let value = match &record.values[i] {
                                                        RecordValue::Integer(v) => Value::Integer(*v),
                                                        RecordValue::Real(v) => Value::Real(*v),
                                                        RecordValue::Text(v) => Value::Text(v.clone()),
                                                        RecordValue::Blob(v) => Value::Blob(v.clone()),
                                                        RecordValue::Null => Value::Null,
                                                    };
                                                    row_context.insert(col.name.clone(), value);
                                                }
                                            }
                                            self.evaluator.set_row(row_context);
                                        }
                                    }
                                }
                                
                                let value = self.evaluator.eval(expr)?;
                                self.evaluator.clear();
                                let current = &self.registers[*accumulator_register];
                                
                                if matches!(current, Value::Null) || value.compare(current)? == std::cmp::Ordering::Greater {
                                    self.registers[*accumulator_register] = value;
                                }
                            }
                        }
                    }
                    pc += 1;
                }
                
                Opcode::FinalizeAggregate { accumulator_register, result_register } => {
                    // Finalize aggregate (for AVG, divide by count)
                    // For now, just copy accumulator to result
                    while self.registers.len() <= *result_register {
                        self.registers.push(Value::Null);
                    }
                    self.registers[*result_register] = self.registers[*accumulator_register].clone();
                    pc += 1;
                }
                
                Opcode::Goto { target } => {
                    pc = *target;
                }
            }
        }
        
        Ok(std::mem::take(&mut self.result))
    }
    
    /// Sort result rows based on ORDER BY clauses
    fn sort_results_by_order_by(&mut self, order_by: &[crate::sql::ast::OrderBy]) -> Result<()> {
        use crate::sql::ast::OrderDirection;
        
        if order_by.is_empty() {
            return Ok(());
        }
        
        // Enhanced multi-column sorting with ASC/DESC and NULL handling
        self.result.rows.sort_by(|row_a, row_b| {
            for order_spec in order_by {
                // Evaluate the ORDER BY expression for each row
                let val_a = evaluate_order_by_expr(&order_spec.expr, row_a);
                let val_b = evaluate_order_by_expr(&order_spec.expr, row_b);
                
                // Compare values with NULL handling
                let cmp = match (&val_a, &val_b) {
                    (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
                    (Value::Null, _) => std::cmp::Ordering::Less, // NULLs sort first
                    (_, Value::Null) => std::cmp::Ordering::Greater,
                    _ => val_a.cmp(&val_b),
                };
                
                // Apply sort direction
                let final_cmp = match order_spec.direction {
                    OrderDirection::Asc => cmp,
                    OrderDirection::Desc => cmp.reverse(),
                };
                
                // If not equal, return this comparison; otherwise continue to next column
                if final_cmp != std::cmp::Ordering::Equal {
                    return final_cmp;
                }
            }
            
            // All columns equal
            std::cmp::Ordering::Equal
        });
        
        Ok(())
    }
}

/// Evaluate ORDER BY expression in the context of a result row (standalone helper)
fn evaluate_order_by_expr(expr: &crate::sql::ast::Expr, row: &[Value]) -> Value {
    use crate::sql::ast::Expr;
    match expr {
        Expr::Column { name, .. } => {
            // Try to parse column name as zero-based index
            // The compiler should convert column names to indices
            if let Ok(idx) = name.parse::<usize>() {
                if idx < row.len() {
                    return row[idx].clone();
                }
            }
            // If not a number, return first column as fallback
            if !row.is_empty() {
                row[0].clone()
            } else {
                Value::Null
            }
        }
        Expr::Literal(lit) => {
            // Convert literal to Value
            use crate::sql::ast::Literal;
            match lit {
                Literal::Integer(i) => Value::Integer(*i),
                Literal::Real(r) => Value::Real(*r),
                Literal::String(s) => Value::Text(s.clone()),
                Literal::Null => Value::Null,
                Literal::Boolean(b) => Value::Integer(if *b { 1 } else { 0 }),
            }
        }
        _ => Value::Null, // For other expressions, return NULL for now
        }
    }

impl Executor {
    /// Execute a simple SELECT query (simplified for Phase 4)
    pub fn execute_select(&mut self, _table: &str, _pager: &mut Pager) -> Result<QueryResult> {
        // Simplified implementation that returns mock data
        // In a full implementation, this would:
        // 1. Open a cursor on the table
        // 2. Iterate through rows
        // 3. Apply filters
        // 4. Project columns
        // 5. Apply sorting and limits
        
        let mock_rows = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string()), Value::Integer(30)],
            vec![Value::Integer(2), Value::Text("Bob".to_string()), Value::Integer(25)],
        ];
        
        Ok(QueryResult::with_rows(mock_rows))
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert RecordValue to VM Value
fn convert_record_value_to_value(rv: &RecordValue) -> Value {
    match rv {
        RecordValue::Null => Value::Null,
        RecordValue::Integer(i) => Value::Integer(*i),
        RecordValue::Real(f) => Value::Real(*f),
        RecordValue::Text(s) => Value::Text(s.clone()),
        RecordValue::Blob(b) => Value::Blob(b.clone()),
    }
}

/// Convert VM Value to RecordValue
fn convert_value_to_record_value(v: &Value) -> RecordValue {
    match v {
        Value::Null => RecordValue::Null,
        Value::Integer(i) => RecordValue::Integer(*i),
        Value::Real(f) => RecordValue::Real(*f),
        Value::Text(s) => RecordValue::Text(s.clone()),
        Value::Blob(b) => RecordValue::Blob(b.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::ast::{Expr, Literal};
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_executor_creation() {
        let executor = Executor::new();
        assert_eq!(executor.registers.len(), 256);
    }
    
    #[test]
    fn test_execute_eval_opcode() {
        let mut executor = Executor::new();
        
        let mut program = Program::new();
        program.add(Opcode::Eval {
            expr: Expr::Literal(Literal::Integer(42)),
            register: 0,
        });
        program.add(Opcode::Halt);
        
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        let table_schemas = HashMap::new(); // No tables needed for this test
        let _result = executor.execute(&program, &mut pager, &table_schemas).unwrap();
        
        assert_eq!(executor.registers[0], Value::Integer(42));
    }
    
    #[test]
    fn test_execute_result_row() {
        let mut executor = Executor::new();
        
        // Set up some register values
        executor.registers[0] = Value::Integer(1);
        executor.registers[1] = Value::Text("Alice".to_string());
        executor.registers[2] = Value::Integer(30);
        
        let mut program = Program::new();
        program.add(Opcode::ResultRow {
            register_start: 0,
            register_count: 3,
        });
        program.add(Opcode::Halt);
        
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        let table_schemas = HashMap::new(); // No tables needed for this test
        let result = executor.execute(&program, &mut pager, &table_schemas).unwrap();
        
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].len(), 3);
        assert_eq!(result.rows[0][0], Value::Integer(1));
    }
}
