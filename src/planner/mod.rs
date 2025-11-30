/// Query Planner - Converts AST to execution plans
/// 
/// Implements logical and physical plan generation

/// Logical plan structures
pub mod logical;

/// Physical plan structures
pub mod physical;

/// Plan builder (AST → Logical Plan)
pub mod builder;

/// Plan optimizer
pub mod optimizer;

pub use logical::*;
pub use physical::*;
pub use builder::PlanBuilder;
pub use optimizer::Optimizer;

