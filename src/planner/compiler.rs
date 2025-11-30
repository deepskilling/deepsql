/// VM Compiler - Converts Physical Plans to VM Opcodes
/// 
/// This module compiles optimized physical query plans into
/// executable VM instruction sequences.

use crate::error::{Error, Result};
use crate::planner::physical::PhysicalPlan;
use crate::vm::opcode::{Opcode, Program};
use crate::sql::ast::{Expr, OrderBy};

/// VM Compiler
/// 
/// Converts physical plans to executable VM opcode sequences
pub struct VMCompiler {
    /// Generated opcodes
    opcodes: Vec<Opcode>,
    
    /// Next cursor ID to allocate
    next_cursor: usize,
    
    /// Next register to allocate
    next_register: usize,
    
    /// Current cursor ID (for the active scan)
    current_cursor: Option<usize>,
    
    /// Current table name (for column resolution)
    current_table: Option<String>,
    
    /// Table schemas for column index lookup
    table_schemas: std::collections::HashMap<String, crate::catalog::schema::TableSchema>,
}

impl VMCompiler {
    /// Create a new VM compiler
    pub fn new() -> Self {
        VMCompiler {
            opcodes: Vec::new(),
            next_cursor: 0,
            next_register: 0,
            current_cursor: None,
            current_table: None,
            table_schemas: std::collections::HashMap::new(),
        }
    }
    
    /// Set table schemas for column resolution
    pub fn set_table_schemas(&mut self, schemas: std::collections::HashMap<String, crate::catalog::schema::TableSchema>) {
        self.table_schemas = schemas;
    }
    
    /// Compile a physical plan to VM opcodes
    pub fn compile(&mut self, plan: &PhysicalPlan) -> Result<Program> {
        // Compile the plan tree
        self.compile_plan(plan)?;
        
        // Add final halt
        let halt_position = self.opcodes.len();
        self.opcodes.push(Opcode::Halt);
        
        // Patch all placeholder jump targets
        self.patch_jump_targets(halt_position);
        
        let program = Program {
            opcodes: std::mem::take(&mut self.opcodes),
        };
        
        #[cfg(test)]
        {
            eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("VM Program: {} opcodes", program.opcodes.len());
            for (i, opcode) in program.opcodes.iter().enumerate() {
                eprintln!("  {}: {:?}", i, opcode);
            }
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        }
        
        Ok(program)
    }
    
    /// Patch placeholder jump targets to point to the correct locations
    fn patch_jump_targets(&mut self, halt_position: usize) {
        for i in 0..self.opcodes.len() {
            match &mut self.opcodes[i] {
                Opcode::Rewind { jump_if_empty, .. } if *jump_if_empty >= 1000 => {
                    // Placeholder value (>= 1000), patch to halt
                    *jump_if_empty = halt_position;
                }
                Opcode::Next { jump_if_done, .. } if *jump_if_done >= 1000 => {
                    // Placeholder value, patch to halt
                    *jump_if_done = halt_position;
                }
                _ => {}
            }
        }
    }
    
    /// Compile a physical plan node
    fn compile_plan(&mut self, plan: &PhysicalPlan) -> Result<()> {
        match plan {
            PhysicalPlan::TableScan { table, .. } => {
                self.compile_table_scan(table)
            }
            
            PhysicalPlan::IndexScan { table, index, .. } => {
                self.compile_index_scan(table, index)
            }
            
            PhysicalPlan::Filter { input, predicate } => {
                self.compile_filter(input, predicate)
            }
            
            PhysicalPlan::Project { input, expressions, .. } => {
                self.compile_project(input, expressions)
            }
            
            PhysicalPlan::Sort { input, order_by } => {
                self.compile_sort(input, order_by)
            }
            
            PhysicalPlan::Limit { input, limit, offset } => {
                self.compile_limit(input, *limit, *offset)
            }
            
            PhysicalPlan::Insert { table, columns, values } => {
                self.compile_insert(table, columns, values)
            }
            
            PhysicalPlan::Update { table, assignments, filter } => {
                self.compile_update(table, assignments, filter)
            }
            
            PhysicalPlan::Delete { table, filter } => {
                self.compile_delete(table, filter)
            }
        }
    }
    
    /// Compile table scan
    fn compile_table_scan(&mut self, table: &str) -> Result<()> {
        let cursor_id = self.next_cursor;
        self.next_cursor += 1;
        self.current_cursor = Some(cursor_id);
        self.current_table = Some(table.to_string());
        
        // Open cursor on table
        self.opcodes.push(Opcode::TableScan {
            table: table.to_string(),
            cursor_id,
        });
        
        // Rewind to start (placeholder jump target will be patched later)
        self.opcodes.push(Opcode::Rewind {
            cursor_id,
            jump_if_empty: 9999, // Placeholder - will be patched to halt_position
        });
        
        // Loop: read rows until done
        let loop_start = self.opcodes.len();
        
        // For each row, emit ResultRow
        // (columns will be added by projection)
        self.opcodes.push(Opcode::ResultRow {
            register_start: 0,
            register_count: 1, // Placeholder
        });
        
        // Next row (placeholder jump target will be patched later)
        self.opcodes.push(Opcode::Next {
            cursor_id,
            jump_if_done: 9999, // Placeholder - will be patched to halt_position
        });
        
        // Jump back to loop start
        self.opcodes.push(Opcode::Goto {
            target: loop_start,
        });
        
        Ok(())
    }
    
    /// Compile index scan
    fn compile_index_scan(&mut self, _table: &str, _index: &str) -> Result<()> {
        // TODO: Implement index scan compilation
        Err(Error::Internal("Index scan compilation not yet implemented".to_string()))
    }
    
    /// Compile filter (WHERE clause)
    fn compile_filter(&mut self, input: &PhysicalPlan, predicate: &Expr) -> Result<()> {
        // Strategy: compile input, then inject filter checks into the loop
        
        // For now, we'll compile the input scan and track where to inject filter
        let start_len = self.opcodes.len();
        self.compile_plan(input)?;
        
        // Find the ResultRow opcode and inject Filter before it
        // Look backwards from end to find ResultRow
        let mut result_row_idx = None;
        for i in (start_len..self.opcodes.len()).rev() {
            if matches!(self.opcodes[i], Opcode::ResultRow { .. }) {
                result_row_idx = Some(i);
                break;
            }
        }
        
        if let Some(idx) = result_row_idx {
            // Insert Filter before ResultRow
            // Filter: if condition is false, jump past ResultRow to Next
            let next_idx = idx + 2; // Jump to the Next instruction
            self.opcodes.insert(idx, Opcode::Filter {
                condition: predicate.clone(),
                jump_target: next_idx,
            });
        }
        
        Ok(())
    }
    
    /// Compile projection (SELECT columns)
    fn compile_project(&mut self, input: &PhysicalPlan, expressions: &[Expr]) -> Result<()> {
        use crate::sql::ast::Expr;
        
        #[cfg(test)]
        eprintln!("DEBUG compile_project: {} expressions", expressions.len());
        
        // First compile input
        let start_len = self.opcodes.len();
        self.compile_plan(input)?;
        
        // Find ResultRow and insert Column opcodes before it
        let mut result_row_idx = None;
        for i in (start_len..self.opcodes.len()).rev() {
            if matches!(self.opcodes[i], Opcode::ResultRow { .. }) {
                result_row_idx = Some(i);
                break;
            }
        }
        
        if let Some(idx) = result_row_idx {
            // Generate Column opcodes for each projected expression
            let mut column_opcodes = Vec::new();
            
            for (reg_idx, expr) in expressions.iter().enumerate() {
                match expr {
                    Expr::Column { table: _, name } => {
                        // Look up column index from table schema
                        let column_index = if let Some(table_name) = &self.current_table {
                            if let Some(schema) = self.table_schemas.get(table_name) {
                                // Find column index by name
                                schema.columns.iter()
                                    .position(|col| &col.name == name)
                                    .unwrap_or(reg_idx) // Fallback to register index
                            } else {
                                reg_idx // No schema, use register index
                            }
                        } else {
                            reg_idx // No current table, use register index
                        };
                        
                        if let Some(cursor_id) = self.current_cursor {
                            column_opcodes.push(Opcode::Column {
                                cursor_id,
                                column_index,
                                register: reg_idx,
                            });
                        }
                    }
                    // Note: Wildcard is handled at the parser level
                    // by converting * to specific columns
                    _ => {
                        // For other expressions, evaluate them
                        column_opcodes.push(Opcode::Eval {
                            expr: expr.clone(),
                            register: reg_idx,
                        });
                    }
                }
            }
            
            // Insert Column opcodes before ResultRow
            for (i, opcode) in column_opcodes.into_iter().enumerate() {
                self.opcodes.insert(idx + i, opcode);
            }
            
            // Update ResultRow to reflect projected columns
            let result_row_new_idx = idx + expressions.len();
            self.opcodes[result_row_new_idx] = Opcode::ResultRow {
                register_start: 0,
                register_count: expressions.len(),
            };
        }
        
        Ok(())
    }
    
    /// Compile sort (ORDER BY)
    fn compile_sort(&mut self, input: &PhysicalPlan, order_by: &[OrderBy]) -> Result<()> {
        // First compile input
        self.compile_plan(input)?;
        
        // Add sort instruction
        self.opcodes.push(Opcode::Sort {
            order_by: order_by.to_vec(),
        });
        
        Ok(())
    }
    
    /// Compile limit (LIMIT/OFFSET)
    fn compile_limit(&mut self, input: &PhysicalPlan, limit: usize, offset: usize) -> Result<()> {
        // First compile input
        self.compile_plan(input)?;
        
        // Add limit instruction
        let counter_register = self.next_register;
        self.next_register += 1;
        
        self.opcodes.push(Opcode::Limit { 
            limit, 
            offset, 
            counter_register,
        });
        
        Ok(())
    }
    
    /// Compile INSERT
    fn compile_insert(&mut self, table: &str, _columns: &Option<Vec<String>>, values: &[Vec<Expr>]) -> Result<()> {
        // Open cursor on table
        let cursor_id = self.next_cursor;
        self.next_cursor += 1;
        
        self.opcodes.push(Opcode::TableScan {
            table: table.to_string(),
            cursor_id,
        });
        
        // For each row to insert
        for row_values in values {
            // Evaluate each value expression and store in registers
            for (reg_idx, expr) in row_values.iter().enumerate() {
                self.opcodes.push(Opcode::Eval {
                    expr: expr.clone(),
                    register: reg_idx,
                });
            }
            
            // Insert the row from registers
            self.opcodes.push(Opcode::Insert {
                cursor_id,
                register_start: 0,
                register_count: row_values.len(),
            });
        }
        
        Ok(())
    }
    
    /// Compile UPDATE
    fn compile_update(&mut self, table: &str, assignments: &[(String, Expr)], filter: &Option<Expr>) -> Result<()> {
        let cursor_id = self.next_cursor;
        self.next_cursor += 1;
        self.current_cursor = Some(cursor_id);
        self.current_table = Some(table.to_string());
        
        // Open cursor on table
        self.opcodes.push(Opcode::TableScan {
            table: table.to_string(),
            cursor_id,
        });
        
        // Rewind to start
        self.opcodes.push(Opcode::Rewind {
            cursor_id,
            jump_if_empty: 9999, // Placeholder
        });
        
        let loop_start = self.opcodes.len();
        
        // Apply filter if present
        if let Some(predicate) = filter {
            // If filter fails, skip this row (jump to Next)
            let next_position = self.opcodes.len() + 100; // Placeholder, will calculate after
            self.opcodes.push(Opcode::Filter {
                condition: predicate.clone(),
                jump_target: next_position,
            });
        }
        
        // Generate Update opcode with assignments
        // Convert assignments to (column_index, expression) pairs
        let update_assignments: Vec<(usize, Expr)> = if let Some(schema) = self.table_schemas.get(table) {
            assignments.iter().map(|(col_name, expr)| {
                let col_idx = schema.columns.iter()
                    .position(|c| &c.name == col_name)
                    .unwrap_or(0);
                (col_idx, expr.clone())
            }).collect()
        } else {
            // Fallback: assume sequential column indices
            assignments.iter().enumerate().map(|(idx, (_, expr))| {
                (idx, expr.clone())
            }).collect()
        };
        
        self.opcodes.push(Opcode::Update {
            cursor_id,
            updates: update_assignments,
        });
        
        // Next row
        self.opcodes.push(Opcode::Next {
            cursor_id,
            jump_if_done: 9999, // Placeholder
        });
        
        // Jump back to loop start
        self.opcodes.push(Opcode::Goto {
            target: loop_start,
        });
        
        Ok(())
    }
    
    /// Compile DELETE
    fn compile_delete(&mut self, table: &str, filter: &Option<Expr>) -> Result<()> {
        let cursor_id = self.next_cursor;
        self.next_cursor += 1;
        self.current_cursor = Some(cursor_id);
        self.current_table = Some(table.to_string());
        
        // Open cursor on table
        self.opcodes.push(Opcode::TableScan {
            table: table.to_string(),
            cursor_id,
        });
        
        // Rewind to start
        self.opcodes.push(Opcode::Rewind {
            cursor_id,
            jump_if_empty: 9999, // Placeholder
        });
        
        let loop_start = self.opcodes.len();
        
        // Apply filter if present
        if let Some(predicate) = filter {
            // If filter fails, skip this row (jump to Next)
            let next_position = self.opcodes.len() + 100; // Placeholder
            self.opcodes.push(Opcode::Filter {
                condition: predicate.clone(),
                jump_target: next_position,
            });
        }
        
        // Delete current row
        self.opcodes.push(Opcode::Delete {
            cursor_id,
        });
        
        // Next row
        self.opcodes.push(Opcode::Next {
            cursor_id,
            jump_if_done: 9999, // Placeholder
        });
        
        // Jump back to loop start
        self.opcodes.push(Opcode::Goto {
            target: loop_start,
        });
        
        Ok(())
    }
}

impl Default for VMCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compiler_creation() {
        let compiler = VMCompiler::new();
        assert_eq!(compiler.opcodes.len(), 0);
    }
}

