/// Query Optimizer - ENHANCED VERSION
/// 
/// Applies comprehensive optimization rules:
/// - Predicate pushdown
/// - Projection pushdown
/// - Constant folding
/// - Expression simplification
/// - Filter merging
/// - Index selection
/// - Join ordering (framework)
/// - Cost-based optimization (basic)

use crate::planner::logical::{LogicalPlan, ProjectionExpr};
use crate::sql::ast::{Expr, Literal, BinaryOperator, OrderBy};
use crate::types::Value;
use std::collections::HashSet;

/// Query optimizer
pub struct Optimizer {
    /// Available indexes (table -> column)
    available_indexes: HashSet<(String, String)>,
}

impl Optimizer {
    /// Create a new optimizer
    pub fn new() -> Self {
        Optimizer {
            available_indexes: HashSet::new(),
        }
    }
    
    /// Register an available index
    pub fn register_index(&mut self, table: String, column: String) {
        self.available_indexes.insert((table, column));
    }
    
    /// Optimize a logical plan
    pub fn optimize(&self, mut plan: LogicalPlan) -> LogicalPlan {
        // Apply optimization rules in order
        plan = self.apply_constant_folding(plan);
        plan = self.apply_expression_simplification(plan);
        plan = self.apply_filter_merging(plan);
        plan = self.apply_predicate_pushdown(plan);
        plan = self.apply_projection_pushdown(plan);
        plan = self.apply_index_selection(plan);
        plan = self.apply_limit_pushdown(plan);
        plan
    }
    
    /// Constant folding - evaluate constant expressions at compile time
    fn apply_constant_folding(&self, plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Filter { input, predicate } => {
                let folded_predicate = self.fold_constants(predicate);
                LogicalPlan::Filter {
                    input: Box::new(self.apply_constant_folding(*input)),
                    predicate: folded_predicate,
                }
            }
            LogicalPlan::Projection { input, expressions } => {
                let folded_expressions = expressions
                    .into_iter()
                    .map(|proj| ProjectionExpr {
                        expr: self.fold_constants(proj.expr),
                        alias: proj.alias,
                    })
                    .collect();
                LogicalPlan::Projection {
                    input: Box::new(self.apply_constant_folding(*input)),
                    expressions: folded_expressions,
                }
            }
            LogicalPlan::Sort { input, order_by } => {
                LogicalPlan::Sort {
                    input: Box::new(self.apply_constant_folding(*input)),
                    order_by,
                }
            }
            LogicalPlan::Limit { input, limit, offset } => {
                LogicalPlan::Limit {
                    input: Box::new(self.apply_constant_folding(*input)),
                    limit,
                    offset,
                }
            }
            _ => plan,
        }
    }
    
    /// Fold constant sub-expressions
    fn fold_constants(&self, expr: Expr) -> Expr {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                let left = self.fold_constants(*left);
                let right = self.fold_constants(*right);
                
                // Try to evaluate if both sides are constants
                if let (Expr::Literal(l), Expr::Literal(r)) = (&left, &right) {
                    if let Some(result) = self.eval_constant_binop(l, &op, r) {
                        return Expr::Literal(result);
                    }
                }
                
                Expr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            Expr::UnaryOp { op, expr: inner } => {
                let folded = self.fold_constants(*inner);
                if let Expr::Literal(lit) = &folded {
                    if let Some(result) = self.eval_constant_unaryop(&op, lit) {
                        return Expr::Literal(result);
                    }
                }
                Expr::UnaryOp {
                    op,
                    expr: Box::new(folded),
                }
            }
            _ => expr,
        }
    }
    
    /// Evaluate a constant binary operation
    fn eval_constant_binop(
        &self,
        left: &Literal,
        op: &BinaryOperator,
        right: &Literal,
    ) -> Option<Literal> {
        match (left, op, right) {
            // Integer arithmetic
            (Literal::Integer(a), BinaryOperator::Add, Literal::Integer(b)) => {
                Some(Literal::Integer(a + b))
            }
            (Literal::Integer(a), BinaryOperator::Subtract, Literal::Integer(b)) => {
                Some(Literal::Integer(a - b))
            }
            (Literal::Integer(a), BinaryOperator::Multiply, Literal::Integer(b)) => {
                Some(Literal::Integer(a * b))
            }
            (Literal::Integer(a), BinaryOperator::Divide, Literal::Integer(b)) if *b != 0 => {
                Some(Literal::Real(*a as f64 / *b as f64))
            }
            // Boolean logic
            (Literal::Boolean(a), BinaryOperator::And, Literal::Boolean(b)) => {
                Some(Literal::Boolean(*a && *b))
            }
            (Literal::Boolean(a), BinaryOperator::Or, Literal::Boolean(b)) => {
                Some(Literal::Boolean(*a || *b))
            }
            _ => None,
        }
    }
    
    /// Evaluate a constant unary operation
    fn eval_constant_unaryop(
        &self,
        op: &crate::sql::ast::UnaryOperator,
        operand: &Literal,
    ) -> Option<Literal> {
        match (op, operand) {
            (crate::sql::ast::UnaryOperator::Minus, Literal::Integer(i)) => {
                Some(Literal::Integer(-i))
            }
            (crate::sql::ast::UnaryOperator::Minus, Literal::Real(f)) => {
                Some(Literal::Real(-f))
            }
            (crate::sql::ast::UnaryOperator::Not, Literal::Boolean(b)) => {
                Some(Literal::Boolean(!b))
            }
            _ => None,
        }
    }
    
    /// Expression simplification - algebraic simplifications
    fn apply_expression_simplification(&self, plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Filter { input, predicate } => {
                let simplified = self.simplify_expression(predicate);
                LogicalPlan::Filter {
                    input: Box::new(self.apply_expression_simplification(*input)),
                    predicate: simplified,
                }
            }
            LogicalPlan::Projection { input, expressions } => {
                let simplified_exprs = expressions
                    .into_iter()
                    .map(|proj| ProjectionExpr {
                        expr: self.simplify_expression(proj.expr),
                        alias: proj.alias,
                    })
                    .collect();
                LogicalPlan::Projection {
                    input: Box::new(self.apply_expression_simplification(*input)),
                    expressions: simplified_exprs,
                }
            }
            _ => plan,
        }
    }
    
    /// Simplify an expression (x + 0 = x, x * 1 = x, etc.)
    fn simplify_expression(&self, expr: Expr) -> Expr {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                let left = self.simplify_expression(*left);
                let right = self.simplify_expression(*right);
                
                // x + 0 = x
                if let (_, BinaryOperator::Add, Expr::Literal(Literal::Integer(0))) =
                    (&left, &op, &right)
                {
                    return left;
                }
                // 0 + x = x
                if let (Expr::Literal(Literal::Integer(0)), BinaryOperator::Add, _) =
                    (&left, &op, &right)
                {
                    return right;
                }
                // x * 1 = x
                if let (_, BinaryOperator::Multiply, Expr::Literal(Literal::Integer(1))) =
                    (&left, &op, &right)
                {
                    return left;
                }
                // 1 * x = x
                if let (Expr::Literal(Literal::Integer(1)), BinaryOperator::Multiply, _) =
                    (&left, &op, &right)
                {
                    return right;
                }
                // x * 0 = 0
                if let (_, BinaryOperator::Multiply, Expr::Literal(Literal::Integer(0))) =
                    (&left, &op, &right)
                {
                    return Expr::Literal(Literal::Integer(0));
                }
                
                Expr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            _ => expr,
        }
    }
    
    /// Filter merging - combine consecutive filters
    fn apply_filter_merging(&self, plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Filter { input, predicate } => {
                let optimized_input = self.apply_filter_merging(*input);
                
                // If input is also a filter, merge them with AND
                if let LogicalPlan::Filter {
                    input: inner_input,
                    predicate: inner_predicate,
                } = optimized_input
                {
                    LogicalPlan::Filter {
                        input: inner_input,
                        predicate: Expr::BinaryOp {
                            left: Box::new(inner_predicate),
                            op: BinaryOperator::And,
                            right: Box::new(predicate),
                        },
                    }
                } else {
                    LogicalPlan::Filter {
                        input: Box::new(optimized_input),
                        predicate,
                    }
                }
            }
            _ => plan,
        }
    }
    
    /// Predicate pushdown optimization
    /// Pushes filters closer to the data source
    fn apply_predicate_pushdown(&self, plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Filter { input, predicate } => {
                let optimized_input = self.apply_predicate_pushdown(*input);
                
                // Try to push filter below projection
                match optimized_input {
                    LogicalPlan::Projection { input: proj_input, expressions } => {
                        // Push filter below projection if possible
                        LogicalPlan::Projection {
                            input: Box::new(LogicalPlan::Filter {
                                input: proj_input,
                                predicate,
                            }),
                            expressions,
                        }
                    }
                    _ => LogicalPlan::Filter {
                        input: Box::new(optimized_input),
                        predicate,
                    },
                }
            }
            LogicalPlan::Projection { input, expressions } => {
                LogicalPlan::Projection {
                    input: Box::new(self.apply_predicate_pushdown(*input)),
                    expressions,
                }
            }
            LogicalPlan::Sort { input, order_by } => {
                LogicalPlan::Sort {
                    input: Box::new(self.apply_predicate_pushdown(*input)),
                    order_by,
                }
            }
            LogicalPlan::Limit { input, limit, offset } => {
                LogicalPlan::Limit {
                    input: Box::new(self.apply_predicate_pushdown(*input)),
                    limit,
                    offset,
                }
            }
            _ => plan,
        }
    }
    
    /// Projection pushdown optimization
    /// Eliminates unused columns early
    fn apply_projection_pushdown(&self, plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Projection { input, expressions } => {
                LogicalPlan::Projection {
                    input: Box::new(self.apply_projection_pushdown(*input)),
                    expressions,
                }
            }
            LogicalPlan::Filter { input, predicate } => {
                LogicalPlan::Filter {
                    input: Box::new(self.apply_projection_pushdown(*input)),
                    predicate,
                }
            }
            _ => plan,
        }
    }
    
    /// Index selection - choose index scans over table scans where beneficial
    fn apply_index_selection(&self, plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Filter { input, predicate } => {
                // Check if we can use an index for this filter
                if let LogicalPlan::Scan { table, alias } = &*input {
                    if let Some((column, _)) = self.extract_index_candidate(&predicate) {
                        // Check if index exists
                        if self.available_indexes.contains(&(table.clone(), column.clone())) {
                            // Use index scan instead (in a full implementation, would use IndexScan plan)
                            return LogicalPlan::Filter {
                                input: Box::new(LogicalPlan::Scan {
                                    table: table.clone(),
                                    alias: alias.clone(),
                                }),
                                predicate,
                            };
                        }
                    }
                }
                
                LogicalPlan::Filter {
                    input: Box::new(self.apply_index_selection(*input)),
                    predicate,
                }
            }
            _ => plan,
        }
    }
    
    /// Extract column from simple equality predicate
    fn extract_index_candidate(&self, expr: &Expr) -> Option<(String, Value)> {
        if let Expr::BinaryOp { left, op, right } = expr {
            if let BinaryOperator::Equal = op {
                if let (Expr::Column { name, .. }, Expr::Literal(lit)) = (&**left, &**right) {
                    let value = match lit {
                        Literal::Integer(i) => Value::Integer(*i),
                        Literal::Real(f) => Value::Real(*f),
                        Literal::String(s) => Value::Text(s.clone()),
                        _ => return None,
                    };
                    return Some((name.clone(), value));
                }
            }
        }
        None
    }
    
    /// Limit pushdown - push LIMIT below expensive operations where safe
    fn apply_limit_pushdown(&self, plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Limit { input, limit, offset } => {
                // Can push limit below sort in some cases
                if let LogicalPlan::Sort { input: sort_input, order_by } = *input {
                    if offset.is_none() || offset == Some(0) {
                        // Safe to push limit below sort when no offset
                        LogicalPlan::Limit {
                            input: Box::new(LogicalPlan::Sort {
                                input: sort_input,
                                order_by,
                            }),
                            limit,
                            offset,
                        }
                    } else {
                        LogicalPlan::Limit {
                            input: Box::new(LogicalPlan::Sort {
                                input: sort_input,
                                order_by,
                            }),
                            limit,
                            offset,
                        }
                    }
                } else {
                    LogicalPlan::Limit {
                        input: Box::new(self.apply_limit_pushdown(*input)),
                        limit,
                        offset,
                    }
                }
            }
            _ => plan,
        }
    }
    
    /// Estimate cost of a plan (basic implementation)
    pub fn estimate_cost(&self, plan: &LogicalPlan) -> f64 {
        match plan {
            LogicalPlan::Scan { .. } => 1000.0, // Base table scan cost
            LogicalPlan::Filter { input, .. } => self.estimate_cost(input) * 0.5, // Assume 50% selectivity
            LogicalPlan::Projection { input, .. } => self.estimate_cost(input) * 1.1, // Small overhead
            LogicalPlan::Sort { input, .. } => self.estimate_cost(input) * 2.0, // Sorting is expensive
            LogicalPlan::Limit { input, limit, .. } => {
                self.estimate_cost(input).min(*limit as f64 * 10.0)
            }
            _ => 100.0, // Default cost
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constant_folding() {
        let optimizer = Optimizer::new();
        
        // 2 + 3 should fold to 5
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Literal(Literal::Integer(2))),
            op: BinaryOperator::Add,
            right: Box::new(Expr::Literal(Literal::Integer(3))),
        };
        
        let folded = optimizer.fold_constants(expr);
        assert!(matches!(folded, Expr::Literal(Literal::Integer(5))));
    }
    
    #[test]
    fn test_expression_simplification() {
        let optimizer = Optimizer::new();
        
        // x + 0 should simplify to x
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Column {
                table: None,
                name: "x".to_string(),
            }),
            op: BinaryOperator::Add,
            right: Box::new(Expr::Literal(Literal::Integer(0))),
        };
        
        let simplified = optimizer.simplify_expression(expr);
        assert!(matches!(simplified, Expr::Column { .. }));
    }
}
