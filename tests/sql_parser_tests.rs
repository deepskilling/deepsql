/// Integration tests for Phase 3: SQL Engine Basics

use deepsql::sql::{Lexer, Parser};
use deepsql::sql::ast::*;

#[test]
fn test_lex_simple_select() {
    let mut lexer = Lexer::new("SELECT * FROM users");
    let tokens = lexer.tokenize();
    
    assert!(tokens.len() >= 5); // SELECT, *, FROM, users, EOF
}

#[test]
fn test_parse_select_star() {
    let mut lexer = Lexer::new("SELECT * FROM users");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert_eq!(select.columns.len(), 1);
            assert!(matches!(select.columns[0], SelectColumn::Star));
            assert_eq!(select.from, Some("users".to_string()));
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_columns() {
    let mut lexer = Lexer::new("SELECT name, age FROM users");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert_eq!(select.columns.len(), 2);
            assert_eq!(select.from, Some("users".to_string()));
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_where() {
    let mut lexer = Lexer::new("SELECT * FROM users WHERE age > 18");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_order_by() {
    let mut lexer = Lexer::new("SELECT * FROM users ORDER BY age DESC");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert_eq!(select.order_by.len(), 1);
            assert_eq!(select.order_by[0].direction, OrderDirection::Desc);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_limit_offset() {
    let mut lexer = Lexer::new("SELECT * FROM users LIMIT 10 OFFSET 20");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert_eq!(select.limit, Some(10));
            assert_eq!(select.offset, Some(20));
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_insert_simple() {
    let mut lexer = Lexer::new("INSERT INTO users VALUES ('Alice', 30)");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Insert(insert) => {
            assert_eq!(insert.table, "users");
            assert_eq!(insert.values.len(), 1);
            assert_eq!(insert.values[0].len(), 2);
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_parse_insert_with_columns() {
    let mut lexer = Lexer::new("INSERT INTO users (name, age) VALUES ('Bob', 25)");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Insert(insert) => {
            assert_eq!(insert.table, "users");
            assert!(insert.columns.is_some());
            assert_eq!(insert.columns.unwrap().len(), 2);
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_parse_insert_multiple_rows() {
    let mut lexer = Lexer::new("INSERT INTO users VALUES ('Alice', 30), ('Bob', 25)");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Insert(insert) => {
            assert_eq!(insert.values.len(), 2);
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_parse_update() {
    let mut lexer = Lexer::new("UPDATE users SET age = 31 WHERE name = 'Alice'");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Update(update) => {
            assert_eq!(update.table, "users");
            assert_eq!(update.assignments.len(), 1);
            assert!(update.where_clause.is_some());
        }
        _ => panic!("Expected UPDATE statement"),
    }
}

#[test]
fn test_parse_update_multiple_columns() {
    let mut lexer = Lexer::new("UPDATE users SET age = 31, name = 'Alicia'");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Update(update) => {
            assert_eq!(update.assignments.len(), 2);
        }
        _ => panic!("Expected UPDATE statement"),
    }
}

#[test]
fn test_parse_delete() {
    let mut lexer = Lexer::new("DELETE FROM users WHERE age < 18");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Delete(delete) => {
            assert_eq!(delete.table, "users");
            assert!(delete.where_clause.is_some());
        }
        _ => panic!("Expected DELETE statement"),
    }
}

#[test]
fn test_parse_delete_all() {
    let mut lexer = Lexer::new("DELETE FROM users");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Delete(delete) => {
            assert_eq!(delete.table, "users");
            assert!(delete.where_clause.is_none());
        }
        _ => panic!("Expected DELETE statement"),
    }
}

#[test]
fn test_parse_create_table_simple() {
    let mut lexer = Lexer::new("CREATE TABLE users (id INTEGER, name TEXT)");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::CreateTable(create) => {
            assert_eq!(create.table, "users");
            assert_eq!(create.columns.len(), 2);
        }
        _ => panic!("Expected CREATE TABLE statement"),
    }
}

#[test]
fn test_parse_create_table_constraints() {
    let mut lexer = Lexer::new("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::CreateTable(create) => {
            assert_eq!(create.columns.len(), 2);
            assert!(!create.columns[0].constraints.is_empty());
            assert!(!create.columns[1].constraints.is_empty());
        }
        _ => panic!("Expected CREATE TABLE statement"),
    }
}

#[test]
fn test_expression_arithmetic() {
    let mut lexer = Lexer::new("SELECT age + 1 FROM users");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert_eq!(select.columns.len(), 1);
            match &select.columns[0] {
                SelectColumn::Expr { expr, .. } => {
                    assert!(matches!(expr, Expr::BinaryOp { .. }));
                }
                _ => panic!("Expected expression column"),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_expression_comparison() {
    let mut lexer = Lexer::new("SELECT * FROM users WHERE age >= 18 AND age <= 65");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_expression_nested() {
    let mut lexer = Lexer::new("SELECT * FROM users WHERE (age > 18 OR age < 5) AND active = 1");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_string_literals() {
    let mut lexer = Lexer::new("INSERT INTO users VALUES ('Alice', 'Bob''s friend')");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Insert(_) => {
            // Successfully parsed
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_null_values() {
    let mut lexer = Lexer::new("INSERT INTO users VALUES (NULL, 'Unknown')");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Insert(insert) => {
            assert_eq!(insert.values[0].len(), 2);
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_comments() {
    let mut lexer = Lexer::new("-- This is a comment\nSELECT * FROM users");
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    let stmt = parser.parse_statement().unwrap();
    
    match stmt {
        Statement::Select(_) => {
            // Successfully parsed, comment was skipped
        }
        _ => panic!("Expected SELECT statement"),
    }
}

