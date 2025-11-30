/// CLI Module
/// 
/// Interactive command-line interface for DeepSQL

/// REPL (Read-Eval-Print Loop)
pub mod repl;

/// Dot commands (.tables, .schema, etc.)
pub mod commands;

/// Pretty table formatting
pub mod formatter;

pub use repl::Repl;
pub use commands::DotCommand;
pub use formatter::Formatter;

