/// Virtual Machine Execution Engine
/// 
/// Opcode-based execution engine for query plans

/// VM opcodes
pub mod opcode;

/// VM executor
pub mod executor;

/// Expression evaluator
pub mod evaluator;

pub use opcode::*;
pub use executor::Executor;
pub use evaluator::ExprEvaluator;

