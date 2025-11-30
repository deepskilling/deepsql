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

/// Query plan cache
pub mod plan_cache;

/// Table statistics for cost-based optimization
pub mod statistics;

pub use logical::*;
pub use physical::*;
pub use builder::PlanBuilder;
pub use optimizer::Optimizer;
pub use plan_cache::PlanCache;
pub use statistics::{StatisticsManager, TableStatistics};

/// VM opcode compiler
pub mod compiler;

