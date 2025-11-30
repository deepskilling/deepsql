/// CREATE INDEX statement AST

/// CREATE INDEX statement structure
#[derive(Debug, Clone, PartialEq)]
pub struct CreateIndexStatement {
    /// Index name
    pub name: String,
    
    /// Table name to create index on
    pub table: String,
    
    /// Columns to index
    pub columns: Vec<String>,
    
    /// Whether this is a UNIQUE index
    pub unique: bool,
}

impl CreateIndexStatement {
    /// Create a new CREATE INDEX statement
    pub fn new(name: String, table: String, columns: Vec<String>) -> Self {
        CreateIndexStatement {
            name,
            table,
            columns,
            unique: false,
        }
    }
    
    /// Mark as unique index
    pub fn with_unique(mut self) -> Self {
        self.unique = true;
        self
    }
}

