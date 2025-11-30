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
pub mod create_index;

pub use expr::*;
pub use select::*;
pub use insert::*;
pub use update::*;
pub use delete::*;
pub use create_table::*;
pub use create_index::*;

/// Top-level SQL statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    CreateIndex(CreateIndexStatement),
    Begin,
    Commit,
    Rollback,
}

