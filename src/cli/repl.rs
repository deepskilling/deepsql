/// REPL - Read-Eval-Print Loop
/// 
/// Interactive SQL shell for DeepSQL

use crate::catalog::CatalogManager;
use crate::cli::commands::DotCommand;
use crate::cli::formatter::Formatter;
use crate::engine::Engine;
use crate::error::Result;
use crate::execution::{SelectExecutor, InsertExecutor, UpdateExecutor, DeleteExecutor};
use crate::planner::builder::PlanBuilder;
use crate::planner::logical::LogicalPlan;
use crate::sql::{Lexer, Parser, Statement};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::time::Instant;

/// REPL state
pub struct Repl {
    /// Database engine
    engine: Engine,
    
    /// Catalog manager
    catalog: CatalogManager,
    
    /// Current database path
    db_path: String,
    
    /// Show execution timing
    show_timing: bool,
}

impl Repl {
    /// Create a new REPL
    pub fn new(db_path: &str) -> Result<Self> {
        let mut engine = Engine::open(db_path)?;
        let mut catalog = CatalogManager::new();
        catalog.load(engine.pager_mut())?;
        
        Ok(Repl {
            engine,
            catalog,
            db_path: db_path.to_string(),
            show_timing: true,
        })
    }
    
    /// Run the REPL
    pub fn run(&mut self) -> Result<()> {
        println!("DeepSQL v0.1.0 - Interactive SQL Shell");
        println!("Type .help for help, .quit to exit\n");
        
        let mut rl: DefaultEditor = DefaultEditor::new()
            .map_err(|e| crate::error::Error::Internal(format!("Failed to initialize readline: {}", e)))?;
        
        // Try to load history
        let history_path = dirs::home_dir()
            .map(|mut p| {
                p.push(".deepsql_history");
                p
            });
        
        if let Some(ref path) = history_path {
            let _ = rl.load_history(path);
        }
        
        let mut buffer = String::new();
        
        loop {
            let prompt = if buffer.is_empty() {
                "deepsql> "
            } else {
                "      -> "
            };
            
            let readline = rl.readline(prompt);
            
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    
                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }
                    
                    // Add to history
                    let _ = rl.add_history_entry(line);
                    
                    // Check for dot command
                    if line.starts_with('.') && buffer.is_empty() {
                        if let Some(cmd) = DotCommand::parse(line) {
                            match cmd {
                                DotCommand::Quit | DotCommand::Exit => {
                                    println!("Goodbye!");
                                    break;
                                }
                                DotCommand::Open(path) => {
                                    match self.open_database(&path) {
                                        Ok(_) => println!("Opened database: {}", path),
                                        Err(e) => println!("{}", Formatter::format_error(&e.to_string())),
                                    }
                                }
                                _ => {
                                    match cmd.execute(&self.catalog) {
                                        Ok(output) => println!("{}", output),
                                        Err(e) => println!("{}", Formatter::format_error(&e.to_string())),
                                    }
                                }
                            }
                        } else {
                            println!("Unknown command. Type .help for help.");
                        }
                        continue;
                    }
                    
                    // Accumulate multi-line SQL
                    buffer.push_str(line);
                    buffer.push(' ');
                    
                    // Check if statement is complete (ends with semicolon)
                    if line.ends_with(';') {
                        let sql = buffer.trim().trim_end_matches(';').to_string();
                        buffer.clear();
                        
                        let start = Instant::now();
                        match self.execute_sql(&sql) {
                            Ok(output) => {
                                println!("{}", output);
                                if self.show_timing {
                                    let elapsed = start.elapsed();
                                    println!("Time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
                                }
                            }
                            Err(e) => {
                                println!("{}", Formatter::format_error(&e.to_string()));
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - cancel current input
                    buffer.clear();
                    println!("^C");
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - exit
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        // Save history
        if let Some(ref path) = history_path {
            let _ = rl.save_history(path);
        }
        
        Ok(())
    }
    
    /// Execute SQL statement
    fn execute_sql(&mut self, sql: &str) -> Result<String> {
        // Parse SQL
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let statement = parser.parse_statement()?;
        
        // Build plan
        let builder = PlanBuilder::new();
        let plan = builder.build(statement.clone())?;
        
        // Execute based on statement type
        match statement {
            Statement::CreateTable(_) => {
                self.catalog.create_table(&plan, self.engine.pager_mut())?;
                self.catalog.save(self.engine.pager_mut())?;
                Ok("Table created successfully.".to_string())
            }
            Statement::Select(_) => {
                let result = SelectExecutor::execute(plan, &self.catalog, self.engine.pager_mut())?;
                
                // Extract column names (simplified - in production, get from schema)
                let column_names = vec!["*".to_string()]; // TODO: Extract real column names
                
                Ok(Formatter::format_result(&result, &column_names))
            }
            Statement::Insert(_) => {
                let result = InsertExecutor::execute(plan, &mut self.catalog, self.engine.pager_mut())?;
                Ok(Formatter::format_result(&result, &[]))
            }
            Statement::Update(_) => {
                let result = UpdateExecutor::execute(plan, &self.catalog, self.engine.pager_mut())?;
                Ok(Formatter::format_result(&result, &[]))
            }
            Statement::Delete(_) => {
                let result = DeleteExecutor::execute(plan, &self.catalog, self.engine.pager_mut())?;
                Ok(Formatter::format_result(&result, &[]))
            }
        }
    }
    
    /// Open a new database
    fn open_database(&mut self, path: &str) -> Result<()> {
        let mut engine = Engine::open(path)?;
        let mut catalog = CatalogManager::new();
        catalog.load(engine.pager_mut())?;
        
        self.engine = engine;
        self.catalog = catalog;
        self.db_path = path.to_string();
        
        Ok(())
    }
}

// Helper to get home directory (for history)
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

