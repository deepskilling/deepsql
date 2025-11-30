/// Abstract Syntax Tree (AST) nodes for SQL

/// Expression nodes
pub mod expr;
/// SELECT statement
pub mod select;
/// INSERT statement
pub mod insert;
/// UPDATE statement
pub mod update;
/// DELETE statement
pub mod delete;
/// CREATE TABLE statement
pub mod create_table;

pub use expr::*;
pub use select::*;
pub use insert::*;
pub use update::*;
pub use delete::*;
pub use create_table::*;

/// Top-level SQL statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// SELECT statement
    Select(SelectStatement),
    
    /// INSERT statement
    Insert(InsertStatement),
    
    /// UPDATE statement
    Update(UpdateStatement),
    
    /// DELETE statement
    Delete(DeleteStatement),
    
    /// CREATE TABLE statement
    CreateTable(CreateTableStatement),
}

