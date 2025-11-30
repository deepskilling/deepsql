/// Expression Evaluator - Evaluates SQL expressions to Values
/// 
/// Evaluates expressions against rows of data

use crate::error::{Error, Result};
use crate::sql::ast::{Expr, Literal, BinaryOperator, UnaryOperator};
use crate::types::Value;
use std::collections::HashMap;

/// Expression evaluator context
pub struct ExprEvaluator {
    /// Current row data (column_name -> value)
    row: HashMap<String, Value>,
}

impl ExprEvaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        ExprEvaluator {
            row: HashMap::new(),
        }
    }
    
    /// Set the current row context
    pub fn set_row(&mut self, row: HashMap<String, Value>) {
        self.row = row;
    }
    
    /// Evaluate an expression
    pub fn eval(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Column { table, name } => self.eval_column(table.as_deref(), name),
            Expr::BinaryOp { left, op, right } => {
                let left_val = self.eval(left)?;
                let right_val = self.eval(right)?;
                self.eval_binary_op(&left_val, *op, &right_val)
            }
            Expr::UnaryOp { op, expr } => {
                let val = self.eval(expr)?;
                self.eval_unary_op(*op, &val)
            }
            Expr::Function { name, args } => {
                self.eval_function(name, args)
            }
        }
    }
    
    fn eval_literal(&self, lit: &Literal) -> Result<Value> {
        match lit {
            Literal::Integer(i) => Ok(Value::Integer(*i)),
            Literal::Real(r) => Ok(Value::Real(*r)),
            Literal::String(s) => Ok(Value::Text(s.clone())),
            Literal::Null => Ok(Value::Null),
            Literal::Boolean(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
        }
    }
    
    fn eval_column(&self, _table: Option<&str>, name: &str) -> Result<Value> {
        self.row.get(name)
            .cloned()
            .ok_or_else(|| Error::Internal(format!("Column not found: {}", name)))
    }
    
    fn eval_binary_op(&self, left: &Value, op: BinaryOperator, right: &Value) -> Result<Value> {
        match op {
            BinaryOperator::Add => left.add(right),
            BinaryOperator::Subtract => left.subtract(right),
            BinaryOperator::Multiply => left.multiply(right),
            BinaryOperator::Divide => left.divide(right),
            BinaryOperator::Modulo => left.modulo(right),
            
            BinaryOperator::Equal => {
                Ok(Value::Integer(if left.compare(right)? == std::cmp::Ordering::Equal { 1 } else { 0 }))
            }
            BinaryOperator::NotEqual => {
                Ok(Value::Integer(if left.compare(right)? != std::cmp::Ordering::Equal { 1 } else { 0 }))
            }
            BinaryOperator::Less => {
                Ok(Value::Integer(if left.compare(right)? == std::cmp::Ordering::Less { 1 } else { 0 }))
            }
            BinaryOperator::LessEqual => {
                let cmp = left.compare(right)?;
                Ok(Value::Integer(if cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal { 1 } else { 0 }))
            }
            BinaryOperator::Greater => {
                Ok(Value::Integer(if left.compare(right)? == std::cmp::Ordering::Greater { 1 } else { 0 }))
            }
            BinaryOperator::GreaterEqual => {
                let cmp = left.compare(right)?;
                Ok(Value::Integer(if cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal { 1 } else { 0 }))
            }
            
            BinaryOperator::And => {
                let left_bool = left.is_truthy()?;
                let right_bool = right.is_truthy()?;
                Ok(Value::Integer(if left_bool && right_bool { 1 } else { 0 }))
            }
            BinaryOperator::Or => {
                let left_bool = left.is_truthy()?;
                let right_bool = right.is_truthy()?;
                Ok(Value::Integer(if left_bool || right_bool { 1 } else { 0 }))
            }
        }
    }
    
    fn eval_unary_op(&self, op: UnaryOperator, val: &Value) -> Result<Value> {
        match op {
            UnaryOperator::Not => val.not(),
            UnaryOperator::Minus => val.negate(),
        }
    }
    
    fn eval_function(&self, name: &str, args: &[Expr]) -> Result<Value> {
        match name.to_uppercase().as_str() {
            "COUNT" => {
                // Simple COUNT(*) implementation
                Ok(Value::Integer(1))
            }
            _ => Err(Error::Internal(format!("Unknown function: {}", name))),
        }
    }
}

impl Default for ExprEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eval_literal() {
        let evaluator = ExprEvaluator::new();
        
        let expr = Expr::Literal(Literal::Integer(42));
        assert_eq!(evaluator.eval(&expr).unwrap(), Value::Integer(42));
        
        let expr = Expr::Literal(Literal::String("hello".to_string()));
        assert_eq!(evaluator.eval(&expr).unwrap(), Value::Text("hello".to_string()));
    }
    
    #[test]
    fn test_eval_arithmetic() {
        let evaluator = ExprEvaluator::new();
        
        // 10 + 5
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Literal(Literal::Integer(10))),
            op: BinaryOperator::Add,
            right: Box::new(Expr::Literal(Literal::Integer(5))),
        };
        
        assert_eq!(evaluator.eval(&expr).unwrap(), Value::Integer(15));
    }
    
    #[test]
    fn test_eval_comparison() {
        let evaluator = ExprEvaluator::new();
        
        // 10 > 5
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Literal(Literal::Integer(10))),
            op: BinaryOperator::Greater,
            right: Box::new(Expr::Literal(Literal::Integer(5))),
        };
        
        assert_eq!(evaluator.eval(&expr).unwrap(), Value::Integer(1)); // true = 1
    }
}

