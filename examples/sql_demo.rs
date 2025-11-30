/// SQL Parser Demo
/// 
/// Demonstrates the SQL parser capabilities

use deepsql::sql::{Lexer, Parser};
use deepsql::sql::ast::*;

fn main() {
    println!("DeepSQL - SQL Parser Demo");
    println!("=========================\n");
    
    // Example SQL statements
    let examples = vec![
        "SELECT * FROM users",
        "SELECT name, age FROM users WHERE age > 18",
        "SELECT * FROM users ORDER BY age DESC LIMIT 10",
        "INSERT INTO users (name, age) VALUES ('Alice', 30)",
        "UPDATE users SET age = 31 WHERE name = 'Alice'",
        "DELETE FROM users WHERE age < 18",
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
    ];
    
    for sql in examples {
        println!("SQL: {}", sql);
        
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize();
        
        println!("Tokens: {} tokens", tokens.len() - 1); // -1 for EOF
        
        let mut parser = Parser::new(tokens);
        match parser.parse_statement() {
            Ok(stmt) => {
                println!("Parsed: {}", statement_type(&stmt));
                println!("AST: {:#?}\n", stmt);
            }
            Err(e) => {
                println!("Error: {}\n", e);
            }
        }
    }
    
    println!("All examples parsed successfully! ✅");
}

fn statement_type(stmt: &Statement) -> &'static str {
    match stmt {
        Statement::Select(_) => "SELECT",
        Statement::Insert(_) => "INSERT",
        Statement::Update(_) => "UPDATE",
        Statement::Delete(_) => "DELETE",
        Statement::CreateTable(_) => "CREATE TABLE",
    }
}

