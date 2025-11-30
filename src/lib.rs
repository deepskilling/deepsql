// DeepSQL - A zero-dependency, high-performance, embedded relational database
//
// Copyright (c) 2025 DeepSQL Contributors
// Licensed under MIT OR Apache-2.0

#![warn(missing_docs)]
#![doc = "DeepSQL - A zero-dependency, high-performance, embedded relational database"]

/// Storage engine components
pub mod storage;

/// Error types for DeepSQL
pub mod error;

pub use error::{Error, Result};

/// Write-Ahead Log (WAL) for ACID transactions
pub mod wal;

/// File-based locking for concurrency
pub mod locking;

/// Transaction context for ACID guarantees
pub mod transaction;

/// SQL Engine - Lexer, Parser, and AST
pub mod sql;

/// Type system (INTEGER, REAL, TEXT, BLOB, NULL)
pub mod types;

/// Query planner (AST → Logical Plan → Physical Plan)
pub mod planner;

/// Virtual Machine Executor
pub mod vm;

/// Catalog & Schema Management
pub mod catalog;

/// Index Support
pub mod index;

/// SQL Execution (INSERT, SELECT, UPDATE, DELETE, ORDER BY, LIMIT)
pub mod execution;

/// CLI (Command-Line Interface)
pub mod cli;

/// Database engine facade
pub mod engine;

pub use engine::Engine;

/// Python bindings (optional)
#[cfg(feature = "python")]
pub mod python;

/// SQL execution engine
pub mod sql_engine;

