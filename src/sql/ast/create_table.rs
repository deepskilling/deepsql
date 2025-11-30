/// CREATE TABLE statement AST

/// CREATE TABLE statement structure
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStatement {
    /// Table name
    pub table: String,
    
    /// Column definitions
    pub columns: Vec<ColumnDef>,
}

/// Column definition
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    /// Column name
    pub name: String,
    
    /// Data type
    pub data_type: DataType,
    
    /// Constraints
    pub constraints: Vec<ColumnConstraint>,
}

/// Data types
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum DataType {
    Integer,
    Real,
    Text,
    Blob,
}

/// Column constraints
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum ColumnConstraint {
    /// NOT NULL
    NotNull,
    
    /// PRIMARY KEY
    PrimaryKey,
    
    /// UNIQUE
    Unique,
    
    /// DEFAULT value
    Default(String),
}

impl CreateTableStatement {
    /// Create a new CREATE TABLE statement
    pub fn new(table: String) -> Self {
        CreateTableStatement {
            table,
            columns: Vec::new(),
        }
    }
}

impl ColumnDef {
    /// Create a new column definition
    pub fn new(name: String, data_type: DataType) -> Self {
        ColumnDef {
            name,
            data_type,
            constraints: Vec::new(),
        }
    }
}

