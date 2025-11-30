/// VM Executor - Executes VM programs
/// 
/// Executes opcode programs against the storage engine

use crate::error::Result;
use crate::storage::pager::Pager;
use crate::types::Value;
use crate::vm::evaluator::ExprEvaluator;
use crate::vm::opcode::{Opcode, Program};

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

/// VM Executor
pub struct Executor {
    /// Virtual machine registers
    registers: Vec<Value>,
    
    /// Expression evaluator
    evaluator: ExprEvaluator,
    
    /// Result accumulator
    result: QueryResult,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Self {
        Executor {
            registers: vec![Value::Null; 256], // 256 registers
            evaluator: ExprEvaluator::new(),
            result: QueryResult::new(),
        }
    }
    
    /// Execute a program
    pub fn execute(&mut self, program: &Program, _pager: &mut Pager) -> Result<QueryResult> {
        let mut pc = 0; // Program counter
        
        // Simple execution loop
        while pc < program.opcodes.len() {
            let opcode = &program.opcodes[pc];
            
            match opcode {
                Opcode::Halt => break,
                
                Opcode::TableScan { .. } => {
                    // TODO: Open cursor on table
                    pc += 1;
                }
                
                Opcode::Rewind {  .. } => {
                    // TODO: Rewind cursor
                    // For now, assume not empty
                    pc += 1;
                }
                
                Opcode::Next { jump_if_done, .. } => {
                    // TODO: Move to next row
                    // For now, jump (simulate end of data)
                    pc = *jump_if_done;
                }
                
                Opcode::Column { column_index, register, .. } => {
                    // TODO: Read column from cursor
                    // For now, store dummy value
                    self.registers[*register] = Value::Integer(*column_index as i64);
                    pc += 1;
                }
                
                Opcode::Eval { expr, register } => {
                    let value = self.evaluator.eval(expr)?;
                    self.registers[*register] = value;
                    pc += 1;
                }
                
                Opcode::Filter { condition, jump_target } => {
                    let value = self.evaluator.eval(condition)?;
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
                
                Opcode::Insert {  .. } => {
                    // TODO: Insert row into table
                    self.result.rows_affected += 1;
                    pc += 1;
                }
                
                Opcode::Update { .. } => {
                    // TODO: Update current row
                    self.result.rows_affected += 1;
                    pc += 1;
                }
                
                Opcode::Delete { .. } => {
                    // TODO: Delete current row
                    self.result.rows_affected += 1;
                    pc += 1;
                }
                
                Opcode::Sort { .. } => {
                    // TODO: Sort result rows
                    pc += 1;
                }
                
                Opcode::Limit { limit, offset, counter_register } => {
                    // Simple limit implementation
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
                
                Opcode::Goto { target } => {
                    pc = *target;
                }
            }
        }
        
        Ok(std::mem::take(&mut self.result))
    }
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::ast::{Expr, Literal};
    
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
        
        let mut pager = Pager::open("test_temp.db").unwrap();
        let _result = executor.execute(&program, &mut pager).unwrap();
        
        assert_eq!(executor.registers[0], Value::Integer(42));
        
        // Cleanup
        std::fs::remove_file("test_temp.db").ok();
    }
}

