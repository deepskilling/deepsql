/// Error types for DeepSQL
/// 
/// Centralized error handling for all database operations

use std::fmt;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error types
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// I/O error
    Io(String),
    
    /// Corruption detected
    Corruption(String),
    
    /// Invalid page
    InvalidPage(String),
    
    /// Record error
    RecordError(String),
    
    /// B+Tree error
    BTreeError(String),
    
    /// WAL error
    WalError(String),
    
    /// Lock error
    LockError(String),
    
    /// Transaction error
    TransactionError(String),
    
    /// SQL parsing error
    ParseError {
        /// The error message
        message: String,
        /// Line number where error occurred
        line: usize,
        /// Column number where error occurred
        column: usize,
    },
    
    /// SQL execution error
    ExecutionError(String),
    
    /// Type error
    TypeError(String),
    
    /// Schema error
    SchemaError(String),
    
    /// Constraint violation
    ConstraintViolation(String),
    
    /// Table not found
    TableNotFound(String),
    
    /// Column not found
    ColumnNotFound(String),
    
    /// Internal error (should not happen)
    Internal(String),
    
    /// Invalid argument
    InvalidArgument(String),
    
    /// Record not found
    NotFound,
}

impl Error {
    /// Create a parse error with position
    pub fn parse_error(message: String, line: usize, column: usize) -> Self {
        Error::ParseError { message, line, column }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(msg) => write!(f, "I/O error: {}", msg),
            Error::Corruption(msg) => write!(f, "Database corruption: {}", msg),
            Error::InvalidPage(msg) => write!(f, "Invalid page: {}", msg),
            Error::RecordError(msg) => write!(f, "Record error: {}", msg),
            Error::BTreeError(msg) => write!(f, "B+Tree error: {}", msg),
            Error::WalError(msg) => write!(f, "WAL error: {}", msg),
            Error::LockError(msg) => write!(f, "Lock error: {}", msg),
            Error::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            Error::ParseError { message, line, column } => {
                write!(f, "Parse error at line {}, column {}: {}", line, column, message)
            }
            Error::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            Error::TypeError(msg) => write!(f, "Type error: {}", msg),
            Error::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            Error::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            Error::TableNotFound(table) => write!(f, "Table not found: {}", table),
            Error::ColumnNotFound(column) => write!(f, "Column not found: {}", column),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
            Error::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            Error::NotFound => write!(f, "Record not found"),
        }
    }
}

impl Error {
    /// Check if this is a not-found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound | Error::TableNotFound(_) | Error::ColumnNotFound(_))
    }
    
    /// Check if this is a constraint violation
    pub fn is_constraint_violation(&self) -> bool {
        matches!(self, Error::ConstraintViolation(_))
    }
}

// Implement From traits for common error conversions
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = Error::TableNotFound("users".to_string());
        assert_eq!(err.to_string(), "Table not found: users");
        
        let err = Error::ParseError {
            message: "Unexpected token".to_string(),
            line: 1,
            column: 5,
        };
        assert_eq!(err.to_string(), "Parse error at line 1, column 5: Unexpected token");
    }
    
    #[test]
    fn test_error_type_checks() {
        let err = Error::TableNotFound("users".to_string());
        assert!(err.is_not_found());
        assert!(!err.is_constraint_violation());
        
        let err = Error::ConstraintViolation("Unique constraint violated".to_string());
        assert!(!err.is_not_found());
        assert!(err.is_constraint_violation());
    }
}

