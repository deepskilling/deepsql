/// DeepSQL CLI Binary
/// 
/// Interactive SQL shell and command-line SQL execution

use clap::Parser;
use deepsql::cli::Repl;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "deepsql")]
#[command(author = "DeepSQL Team")]
#[command(version = "0.1.0")]
#[command(about = "DeepSQL - Embedded SQL Database", long_about = None)]
struct Args {
    /// Database file to open
    #[arg(default_value = "deepsql.db")]
    database: String,
    
    /// Execute SQL from command line and exit
    #[arg(short = 'c', long = "command")]
    command: Option<String>,
    
    /// SQL file to execute
    #[arg(short = 'f', long = "file")]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    // If command provided, execute and exit
    if let Some(sql) = args.command {
        if let Err(e) = execute_command(&args.database, &sql) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }
    
    // If file provided, execute file and exit
    if let Some(file_path) = args.file {
        if let Err(e) = execute_file(&args.database, &file_path) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }
    
    // Otherwise, start REPL
    match Repl::new(&args.database) {
        Ok(mut repl) => {
            if let Err(e) = repl.run() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            process::exit(1);
        }
    }
}

fn execute_command(db_path: &str, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repl = Repl::new(db_path)?;
    // Note: This is a simplified version. In a full implementation,
    // we'd expose execute_sql as public and call it directly.
    println!("Command execution not yet implemented. Use REPL mode.");
    Ok(())
}

fn execute_file(db_path: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _sql = std::fs::read_to_string(file_path)?;
    let mut _repl = Repl::new(db_path)?;
    // Note: This is a simplified version. In a full implementation,
    // we'd parse and execute each statement in the file.
    println!("File execution not yet implemented. Use REPL mode.");
    Ok(())
}

