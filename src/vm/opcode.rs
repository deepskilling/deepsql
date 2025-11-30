/// VM Opcodes - Low-level execution instructions
/// 
/// Defines the instruction set for the query execution VM

use crate::sql::ast::{Expr, OrderBy};

/// VM Opcode
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum Opcode {
    /// Open a cursor on a table
    /// TableScan(table_name, cursor_id)
    TableScan {
        table: String,
        cursor_id: usize,
    },
    
    /// Rewind a cursor to the beginning
    /// Rewind(cursor_id, jump_if_empty)
    Rewind {
        cursor_id: usize,
        jump_if_empty: usize,
    },
    
    /// Move cursor to next row
    /// Next(cursor_id, jump_if_done)
    Next {
        cursor_id: usize,
        jump_if_done: usize,
    },
    
    /// Read a column from current cursor position
    /// Column(cursor_id, column_index, register)
    Column {
        cursor_id: usize,
        column_index: usize,
        register: usize,
    },
    
    /// Evaluate an expression and store in register
    /// Eval(expression, register)
    Eval {
        expr: Expr,
        register: usize,
    },
    
    /// Filter: jump if condition is false
    /// Filter(expression, jump_target)
    Filter {
        condition: Expr,
        jump_target: usize,
    },
    
    /// Make a result row from registers
    /// ResultRow(register_start, register_count)
    ResultRow {
        register_start: usize,
        register_count: usize,
    },
    
    /// Insert a row
    /// Insert(cursor_id, register_start, register_count)
    Insert {
        cursor_id: usize,
        register_start: usize,
        register_count: usize,
    },
    
    /// Update current row
    /// Update(cursor_id, updates)
    Update {
        cursor_id: usize,
        updates: Vec<(usize, Expr)>, // (column_index, new_value_expr)
    },
    
    /// Delete current row
    /// Delete(cursor_id)
    Delete {
        cursor_id: usize,
    },
    
    /// Sort rows
    /// Sort(order_by_expressions)
    Sort {
        order_by: Vec<OrderBy>,
    },
    
    /// Limit results
    /// Limit(limit, offset, counter_register)
    Limit {
        limit: usize,
        offset: usize,
        counter_register: usize,
    },
    
    /// Jump to instruction
    /// Goto(target)
    Goto {
        target: usize,
    },
    
    /// Aggregate accumulator - accumulates values across rows
    /// Aggregate(function, expr, accumulator_register)
    Aggregate {
        function: AggregateFunction,
        expr: Option<Expr>, // None for COUNT(*)
        accumulator_register: usize,
    },
    
    /// Finalize aggregate and store result
    /// FinalizeAggregate(accumulator_register, result_register)
    FinalizeAggregate {
        accumulator_register: usize,
        result_register: usize,
    },
    
    /// Halt execution
    Halt,
}

/// Aggregate function types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// VM Program - sequence of opcodes
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Program {
    pub opcodes: Vec<Opcode>,
}

impl Program {
    /// Create a new empty program
    pub fn new() -> Self {
        Program {
            opcodes: Vec::new(),
        }
    }
    
    /// Add an opcode to the program
    pub fn add(&mut self, opcode: Opcode) -> usize {
        let pc = self.opcodes.len();
        self.opcodes.push(opcode);
        pc
    }
    
    /// Get the current program counter (next instruction position)
    pub fn pc(&self) -> usize {
        self.opcodes.len()
    }
    
    /// Patch a Goto or jump target
    pub fn patch(&mut self, position: usize, target: usize) {
        match &mut self.opcodes[position] {
            Opcode::Goto { target: t } => *t = target,
            Opcode::Rewind { jump_if_empty, .. } => *jump_if_empty = target,
            Opcode::Next { jump_if_done, .. } => *jump_if_done = target,
            Opcode::Filter { jump_target, .. } => *jump_target = target,
            _ => panic!("Cannot patch non-jump opcode"),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::TableScan { table, cursor_id } => {
                write!(f, "TableScan {} -> cursor[{}]", table, cursor_id)
            }
            Opcode::Rewind { cursor_id, jump_if_empty } => {
                write!(f, "Rewind cursor[{}] (empty? -> {})", cursor_id, jump_if_empty)
            }
            Opcode::Next { cursor_id, jump_if_done } => {
                write!(f, "Next cursor[{}] (done? -> {})", cursor_id, jump_if_done)
            }
            Opcode::Column { cursor_id, column_index, register } => {
                write!(f, "Column cursor[{}][{}] -> r[{}]", cursor_id, column_index, register)
            }
            Opcode::Eval { register, .. } => {
                write!(f, "Eval -> r[{}]", register)
            }
            Opcode::Filter { jump_target, .. } => {
                write!(f, "Filter (false? -> {})", jump_target)
            }
            Opcode::ResultRow { register_start, register_count } => {
                write!(f, "ResultRow r[{}..{}]", register_start, register_start + register_count)
            }
            Opcode::Insert { cursor_id, register_start, register_count } => {
                write!(f, "Insert cursor[{}] from r[{}..{}]", cursor_id, register_start, register_start + register_count)
            }
            Opcode::Update { cursor_id, .. } => {
                write!(f, "Update cursor[{}]", cursor_id)
            }
            Opcode::Delete { cursor_id } => {
                write!(f, "Delete cursor[{}]", cursor_id)
            }
            Opcode::Sort { .. } => {
                write!(f, "Sort")
            }
            Opcode::Limit { limit, offset, .. } => {
                write!(f, "Limit {} OFFSET {}", limit, offset)
            }
            Opcode::Goto { target } => {
                write!(f, "Goto {}", target)
            }
            Opcode::Aggregate { function, accumulator_register, .. } => {
                write!(f, "Aggregate {:?} -> r[{}]", function, accumulator_register)
            }
            Opcode::FinalizeAggregate { accumulator_register, result_register } => {
                write!(f, "FinalizeAggregate r[{}] -> r[{}]", accumulator_register, result_register)
            }
            Opcode::Halt => {
                write!(f, "Halt")
            }
        }
    }
}

