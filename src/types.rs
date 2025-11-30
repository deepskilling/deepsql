/// Type System for DeepSQL
/// 
/// Implements the core data types: INTEGER, REAL, TEXT, BLOB, NULL

use crate::error::{Error, Result};
use std::cmp::Ordering;

/// SQL Value types
#[derive(Debug, Clone)]
pub enum Value {
    /// NULL value
    Null,
    
    /// INTEGER (64-bit signed integer)
    Integer(i64),
    
    /// REAL (64-bit floating point)
    Real(f64),
    
    /// TEXT (UTF-8 string)
    Text(String),
    
    /// BLOB (binary data)
    Blob(Vec<u8>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.compare(other), Ok(Ordering::Equal))
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.compare(other).ok()
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other).unwrap_or(Ordering::Equal)
    }
}

impl Value {
    /// Get the type of this value
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Integer(_) => ValueType::Integer,
            Value::Real(_) => ValueType::Real,
            Value::Text(_) => ValueType::Text,
            Value::Blob(_) => ValueType::Blob,
        }
    }
    
    /// Check if value is NULL
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    
    /// Convert to integer (with type coercion)
    pub fn to_integer(&self) -> Result<i64> {
        match self {
            Value::Null => Err(Error::Internal("Cannot convert NULL to integer".to_string())),
            Value::Integer(i) => Ok(*i),
            Value::Real(f) => Ok(*f as i64),
            Value::Text(s) => s.parse::<i64>()
                .map_err(|_| Error::Internal(format!("Cannot convert '{}' to integer", s))),
            Value::Blob(_) => Err(Error::Internal("Cannot convert BLOB to integer".to_string())),
        }
    }
    
    /// Convert to real (with type coercion)
    pub fn to_real(&self) -> Result<f64> {
        match self {
            Value::Null => Err(Error::Internal("Cannot convert NULL to real".to_string())),
            Value::Integer(i) => Ok(*i as f64),
            Value::Real(f) => Ok(*f),
            Value::Text(s) => s.parse::<f64>()
                .map_err(|_| Error::Internal(format!("Cannot convert '{}' to real", s))),
            Value::Blob(_) => Err(Error::Internal("Cannot convert BLOB to real".to_string())),
        }
    }
    
    /// Convert to text
    pub fn to_text(&self) -> String {
        match self {
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Real(f) => f.to_string(),
            Value::Text(s) => s.clone(),
            Value::Blob(b) => format!("BLOB({} bytes)", b.len()),
        }
    }
    
    /// Compare two values
    pub fn compare(&self, other: &Value) -> Result<Ordering> {
        match (self, other) {
            // NULL comparisons always return false (SQL semantics)
            (Value::Null, _) | (_, Value::Null) => Ok(Ordering::Equal),
            
            // Integer comparisons
            (Value::Integer(a), Value::Integer(b)) => Ok(a.cmp(b)),
            (Value::Integer(a), Value::Real(b)) => Ok((*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal)),
            
            // Real comparisons
            (Value::Real(a), Value::Integer(b)) => Ok(a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal)),
            (Value::Real(a), Value::Real(b)) => Ok(a.partial_cmp(b).unwrap_or(Ordering::Equal)),
            
            // Text comparisons
            (Value::Text(a), Value::Text(b)) => Ok(a.cmp(b)),
            
            // BLOB comparisons
            (Value::Blob(a), Value::Blob(b)) => Ok(a.cmp(b)),
            
            // Mixed type comparisons
            _ => Err(Error::Internal(format!(
                "Cannot compare {} with {}",
                self.value_type(),
                other.value_type()
            ))),
        }
    }
    
    /// Add two values
    pub fn add(&self, other: &Value) -> Result<Value> {
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(*a as f64 + b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a + *b as f64)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a + b)),
            _ => Err(Error::Internal(format!(
                "Cannot add {} and {}",
                self.value_type(),
                other.value_type()
            ))),
        }
    }
    
    /// Subtract two values
    pub fn subtract(&self, other: &Value) -> Result<Value> {
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(*a as f64 - b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a - *b as f64)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a - b)),
            _ => Err(Error::Internal(format!(
                "Cannot subtract {} and {}",
                self.value_type(),
                other.value_type()
            ))),
        }
    }
    
    /// Multiply two values
    pub fn multiply(&self, other: &Value) -> Result<Value> {
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(*a as f64 * b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a * *b as f64)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a * b)),
            _ => Err(Error::Internal(format!(
                "Cannot multiply {} and {}",
                self.value_type(),
                other.value_type()
            ))),
        }
    }
    
    /// Divide two values
    pub fn divide(&self, other: &Value) -> Result<Value> {
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (_, Value::Integer(0)) | (_, Value::Real(0.0)) => {
                Err(Error::Internal("Division by zero".to_string()))
            }
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Real(*a as f64 / *b as f64)),
            (Value::Integer(a), Value::Real(b)) => Ok(Value::Real(*a as f64 / b)),
            (Value::Real(a), Value::Integer(b)) => Ok(Value::Real(a / *b as f64)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a / b)),
            _ => Err(Error::Internal(format!(
                "Cannot divide {} and {}",
                self.value_type(),
                other.value_type()
            ))),
        }
    }
    
    /// Modulo operation
    pub fn modulo(&self, other: &Value) -> Result<Value> {
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => Ok(Value::Null),
            (_, Value::Integer(0)) => Err(Error::Internal("Modulo by zero".to_string())),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
            _ => Err(Error::Internal(format!(
                "Cannot modulo {} and {}",
                self.value_type(),
                other.value_type()
            ))),
        }
    }
    
    /// Negate a value
    pub fn negate(&self) -> Result<Value> {
        match self {
            Value::Null => Ok(Value::Null),
            Value::Integer(i) => Ok(Value::Integer(-i)),
            Value::Real(f) => Ok(Value::Real(-f)),
            _ => Err(Error::Internal(format!(
                "Cannot negate {}",
                self.value_type()
            ))),
        }
    }
    
    /// Logical NOT
    pub fn not(&self) -> Result<Value> {
        Ok(Value::Integer(if self.is_truthy()? { 0 } else { 1 }))
    }
    
    /// Check if value is truthy (for boolean operations)
    pub fn is_truthy(&self) -> Result<bool> {
        match self {
            Value::Null => Ok(false),
            Value::Integer(i) => Ok(*i != 0),
            Value::Real(f) => Ok(*f != 0.0),
            Value::Text(s) => Ok(!s.is_empty()),
            Value::Blob(b) => Ok(!b.is_empty()),
        }
    }
}

/// Value type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// NULL type
    Null,
    /// INTEGER type
    Integer,
    /// REAL (floating point) type
    Real,
    /// TEXT (string) type
    Text,
    /// BLOB (binary) type
    Blob,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Null => write!(f, "NULL"),
            ValueType::Integer => write!(f, "INTEGER"),
            ValueType::Real => write!(f, "REAL"),
            ValueType::Text => write!(f, "TEXT"),
            ValueType::Blob => write!(f, "BLOB"),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Real(r) => write!(f, "{}", r),
            Value::Text(s) => write!(f, "{}", s),
            Value::Blob(b) => write!(f, "BLOB({} bytes)", b.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_types() {
        assert_eq!(Value::Null.value_type(), ValueType::Null);
        assert_eq!(Value::Integer(42).value_type(), ValueType::Integer);
        assert_eq!(Value::Real(3.14).value_type(), ValueType::Real);
        assert_eq!(Value::Text("hello".to_string()).value_type(), ValueType::Text);
        assert_eq!(Value::Blob(vec![1, 2, 3]).value_type(), ValueType::Blob);
    }
    
    #[test]
    fn test_arithmetic_operations() {
        let a = Value::Integer(10);
        let b = Value::Integer(3);
        
        assert_eq!(a.add(&b).unwrap(), Value::Integer(13));
        assert_eq!(a.subtract(&b).unwrap(), Value::Integer(7));
        assert_eq!(a.multiply(&b).unwrap(), Value::Integer(30));
        assert_eq!(a.modulo(&b).unwrap(), Value::Integer(1));
    }
    
    #[test]
    fn test_type_coercion() {
        let int_val = Value::Integer(42);
        let real_val = Value::Real(3.14);
        
        // Integer + Real = Real
        let result = int_val.add(&real_val).unwrap();
        assert!(matches!(result, Value::Real(_)));
    }
    
    #[test]
    fn test_comparisons() {
        let a = Value::Integer(10);
        let b = Value::Integer(20);
        
        assert_eq!(a.compare(&b).unwrap(), Ordering::Less);
        assert_eq!(b.compare(&a).unwrap(), Ordering::Greater);
        assert_eq!(a.compare(&a).unwrap(), Ordering::Equal);
    }
    
    #[test]
    fn test_null_handling() {
        let null = Value::Null;
        let int_val = Value::Integer(42);
        
        assert!(null.is_null());
        assert!(!int_val.is_null());
        
        // NULL in arithmetic returns NULL
        assert_eq!(null.add(&int_val).unwrap(), Value::Null);
    }
    
    #[test]
    fn test_truthiness() {
        assert!(!Value::Null.is_truthy().unwrap());
        assert!(!Value::Integer(0).is_truthy().unwrap());
        assert!(Value::Integer(1).is_truthy().unwrap());
        assert!(Value::Integer(-1).is_truthy().unwrap());
        assert!(!Value::Real(0.0).is_truthy().unwrap());
        assert!(Value::Real(3.14).is_truthy().unwrap());
    }
}

