/// Schema Structures
/// 
/// Defines database catalog metadata structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database catalog - holds all schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    /// All tables in the database
    pub tables: HashMap<String, TableSchema>,
    
    /// All indexes in the database
    pub indexes: HashMap<String, IndexSchema>,
}

impl Catalog {
    /// Create a new empty catalog
    pub fn new() -> Self {
        Catalog {
            tables: HashMap::new(),
            indexes: HashMap::new(),
        }
    }
    
    /// Add a table to the catalog
    pub fn add_table(&mut self, table: TableSchema) {
        self.tables.insert(table.name.clone(), table);
    }
    
    /// Get a table by name
    pub fn get_table(&self, name: &str) -> Option<&TableSchema> {
        self.tables.get(name)
    }
    
    /// Get a mutable reference to a table
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut TableSchema> {
        self.tables.get_mut(name)
    }
    
    /// Remove a table
    pub fn remove_table(&mut self, name: &str) -> Option<TableSchema> {
        self.tables.remove(name)
    }
    
    /// Add an index to the catalog
    pub fn add_index(&mut self, index: IndexSchema) {
        self.indexes.insert(index.name.clone(), index);
    }
    
    /// Get an index by name
    pub fn get_index(&self, name: &str) -> Option<&IndexSchema> {
        self.indexes.get(name)
    }
    
    /// Remove an index
    pub fn remove_index(&mut self, name: &str) -> Option<IndexSchema> {
        self.indexes.remove(name)
    }
    
    /// List all table names
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Table schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table name
    pub name: String,
    
    /// Root page ID in the database file
    pub root_page: u32,
    
    /// Columns in the table
    pub columns: Vec<ColumnSchema>,
    
    /// Primary key column index (if any)
    pub primary_key: Option<usize>,
}

impl TableSchema {
    /// Create a new table schema
    pub fn new(name: String, root_page: u32) -> Self {
        TableSchema {
            name,
            root_page,
            columns: Vec::new(),
            primary_key: None,
        }
    }
    
    /// Add a column to the table
    pub fn add_column(&mut self, column: ColumnSchema) {
        if column.primary_key {
            self.primary_key = Some(self.columns.len());
        }
        self.columns.push(column);
    }
    
    /// Get column by name
    pub fn get_column(&self, name: &str) -> Option<&ColumnSchema> {
        self.columns.iter().find(|c| c.name == name)
    }
    
    /// Get column index by name
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }
    
    /// Get column by index
    pub fn get_column_by_index(&self, index: usize) -> Option<&ColumnSchema> {
        self.columns.get(index)
    }
}

/// Column schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    /// Column name
    pub name: String,
    
    /// Column data type
    pub data_type: ColumnType,
    
    /// Whether NULL values are allowed
    pub nullable: bool,
    
    /// Whether this is the primary key
    pub primary_key: bool,
    
    /// Whether this column has a UNIQUE constraint
    pub unique: bool,
    
    /// Default value (serialized as string)
    pub default_value: Option<String>,
}

impl ColumnSchema {
    /// Create a new column schema
    pub fn new(name: String, data_type: ColumnType) -> Self {
        ColumnSchema {
            name,
            data_type,
            nullable: true,
            primary_key: false,
            unique: false,
            default_value: None,
        }
    }
    
    /// Set as primary key
    pub fn with_primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false; // Primary key implies NOT NULL
        self
    }
    
    /// Set as not null
    pub fn with_not_null(mut self) -> Self {
        self.nullable = false;
        self
    }
    
    /// Set as unique
    pub fn with_unique(mut self) -> Self {
        self.unique = true;
        self
    }
    
    /// Set default value
    pub fn with_default(mut self, value: String) -> Self {
        self.default_value = Some(value);
        self
    }
}

/// Column data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    /// INTEGER type
    Integer,
    
    /// REAL (floating point) type
    Real,
    
    /// TEXT (string) type
    Text,
    
    /// BLOB (binary) type
    Blob,
}

impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Integer => write!(f, "INTEGER"),
            ColumnType::Real => write!(f, "REAL"),
            ColumnType::Text => write!(f, "TEXT"),
            ColumnType::Blob => write!(f, "BLOB"),
        }
    }
}

/// Index schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSchema {
    /// Index name
    pub name: String,
    
    /// Table this index belongs to
    pub table_name: String,
    
    /// Root page ID in the database file
    pub root_page: u32,
    
    /// Columns included in the index
    pub columns: Vec<String>,
    
    /// Whether this is a unique index
    pub unique: bool,
}

impl IndexSchema {
    /// Create a new index schema
    pub fn new(name: String, table_name: String, root_page: u32) -> Self {
        IndexSchema {
            name,
            table_name,
            root_page,
            columns: Vec::new(),
            unique: false,
        }
    }
    
    /// Add a column to the index
    pub fn add_column(&mut self, column: String) {
        self.columns.push(column);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_catalog_creation() {
        let catalog = Catalog::new();
        assert_eq!(catalog.tables.len(), 0);
        assert_eq!(catalog.indexes.len(), 0);
    }
    
    #[test]
    fn test_table_schema() {
        let mut table = TableSchema::new("users".to_string(), 1);
        
        let col1 = ColumnSchema::new("id".to_string(), ColumnType::Integer)
            .with_primary_key();
        let col2 = ColumnSchema::new("name".to_string(), ColumnType::Text)
            .with_not_null();
        
        table.add_column(col1);
        table.add_column(col2);
        
        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.primary_key, Some(0));
        assert_eq!(table.get_column("name").unwrap().data_type, ColumnType::Text);
    }
    
    #[test]
    fn test_catalog_operations() {
        let mut catalog = Catalog::new();
        
        let mut table = TableSchema::new("users".to_string(), 1);
        table.add_column(ColumnSchema::new("id".to_string(), ColumnType::Integer));
        
        catalog.add_table(table);
        
        assert_eq!(catalog.table_names().len(), 1);
        assert!(catalog.get_table("users").is_some());
        
        catalog.remove_table("users");
        assert_eq!(catalog.table_names().len(), 0);
    }
}

