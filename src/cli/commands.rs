/// Dot Commands
/// 
/// Special commands starting with '.' for database introspection

use crate::catalog::CatalogManager;
use crate::error::Result;

/// Dot command type
#[derive(Debug, Clone, PartialEq)]
pub enum DotCommand {
    /// .tables - List all tables
    Tables,
    
    /// .schema <table> - Show table schema
    Schema(Option<String>),
    
    /// .open <database> - Open a database file
    Open(String),
    
    /// .help - Show help
    Help,
    
    /// .quit - Exit the shell
    Quit,
    
    /// .exit - Exit the shell (alias for .quit)
    Exit,
}

impl DotCommand {
    /// Parse a dot command from input
    pub fn parse(input: &str) -> Option<DotCommand> {
        let input = input.trim();
        if !input.starts_with('.') {
            return None;
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        
        match parts[0].to_lowercase().as_str() {
            ".tables" => Some(DotCommand::Tables),
            ".schema" => {
                let table = parts.get(1).map(|s| s.to_string());
                Some(DotCommand::Schema(table))
            }
            ".open" => {
                if parts.len() < 2 {
                    return None;
                }
                Some(DotCommand::Open(parts[1].to_string()))
            }
            ".help" => Some(DotCommand::Help),
            ".quit" => Some(DotCommand::Quit),
            ".exit" => Some(DotCommand::Exit),
            _ => None,
        }
    }
    
    /// Execute a dot command
    pub fn execute(&self, catalog: &CatalogManager) -> Result<String> {
        match self {
            DotCommand::Tables => {
                let tables = catalog.list_tables();
                if tables.is_empty() {
                    Ok("No tables found.".to_string())
                } else {
                    Ok(tables.join("\n"))
                }
            }
            DotCommand::Schema(table_name) => {
                if let Some(name) = table_name {
                    if let Some(table) = catalog.get_table(name) {
                        let mut output = format!("CREATE TABLE {} (\n", table.name);
                        for (i, col) in table.columns.iter().enumerate() {
                            let type_str = match col.data_type {
                                crate::catalog::schema::ColumnType::Integer => "INTEGER",
                                crate::catalog::schema::ColumnType::Real => "REAL",
                                crate::catalog::schema::ColumnType::Text => "TEXT",
                                crate::catalog::schema::ColumnType::Blob => "BLOB",
                            };
                            
                            output.push_str(&format!("  {} {}", col.name, type_str));
                            
                            // Simple schema display (primary key info is stored at table level)
                            if table.primary_key == Some(i) {
                                output.push_str(" PRIMARY KEY");
                            }
                            
                            if i < table.columns.len() - 1 {
                                output.push(',');
                            }
                            output.push('\n');
                        }
                        output.push_str(");");
                        Ok(output)
                    } else {
                        Ok(format!("Table '{}' not found.", name))
                    }
                } else {
                    // Show all schemas
                    let tables = catalog.list_tables();
                    if tables.is_empty() {
                        Ok("No tables found.".to_string())
                    } else {
                        let mut output = String::new();
                        for table_name in tables {
                            if let Some(table) = catalog.get_table(&table_name) {
                                output.push_str(&format!("CREATE TABLE {} (...)\n", table.name));
                            }
                        }
                        Ok(output)
                    }
                }
            }
            DotCommand::Help => {
                Ok(Self::help_text())
            }
            DotCommand::Quit | DotCommand::Exit => {
                Ok("Goodbye!".to_string())
            }
            DotCommand::Open(_) => {
                Ok("Database switching will be handled by REPL.".to_string())
            }
        }
    }
    
    /// Get help text
    pub fn help_text() -> String {
        r#"DeepSQL Shell - Available Commands:

SQL Statements:
  SELECT ...              Execute a SELECT query
  INSERT INTO ...         Insert data into a table
  UPDATE ...              Update table data
  DELETE FROM ...         Delete data from a table
  CREATE TABLE ...        Create a new table

Dot Commands:
  .tables                 List all tables
  .schema                 Show schema for all tables
  .schema <table>         Show schema for a specific table
  .open <database>        Open a database file
  .help                   Show this help message
  .quit or .exit          Exit the shell

Tips:
  - SQL statements must end with semicolon (;)
  - Dot commands don't need semicolons
  - Use Ctrl+C to cancel current input
  - Use Ctrl+D to exit
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_tables() {
        let cmd = DotCommand::parse(".tables");
        assert_eq!(cmd, Some(DotCommand::Tables));
    }
    
    #[test]
    fn test_parse_schema() {
        let cmd = DotCommand::parse(".schema users");
        assert_eq!(cmd, Some(DotCommand::Schema(Some("users".to_string()))));
        
        let cmd = DotCommand::parse(".schema");
        assert_eq!(cmd, Some(DotCommand::Schema(None)));
    }
    
    #[test]
    fn test_parse_open() {
        let cmd = DotCommand::parse(".open test.db");
        assert_eq!(cmd, Some(DotCommand::Open("test.db".to_string())));
    }
    
    #[test]
    fn test_parse_help() {
        let cmd = DotCommand::parse(".help");
        assert_eq!(cmd, Some(DotCommand::Help));
    }
    
    #[test]
    fn test_parse_quit() {
        let cmd = DotCommand::parse(".quit");
        assert_eq!(cmd, Some(DotCommand::Quit));
        
        let cmd = DotCommand::parse(".exit");
        assert_eq!(cmd, Some(DotCommand::Exit));
    }
    
    #[test]
    fn test_parse_invalid() {
        let cmd = DotCommand::parse(".invalid");
        assert_eq!(cmd, None);
        
        let cmd = DotCommand::parse("SELECT * FROM users");
        assert_eq!(cmd, None);
    }
}

