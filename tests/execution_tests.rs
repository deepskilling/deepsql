/// End-to-End Execution Tests for Phase 7
/// 
/// Tests the complete SQL execution flow: Parse → Plan → Execute

use deepsql::catalog::CatalogManager;
use deepsql::execution::*;
use deepsql::planner::builder::PlanBuilder;
use deepsql::planner::logical::{ColumnSpec, DataType, LogicalPlan};
use deepsql::sql::ast::{Expr, Literal, OrderDirection};
use deepsql::sql::{Lexer, Parser};
use deepsql::storage::pager::Pager;
use deepsql::types::Value;

#[test]
fn test_end_to_end_create_table() {
    let mut pager = Pager::open("test_e2e_create.db").unwrap();
    let mut catalog = CatalogManager::new();
    catalog.load(&mut pager).unwrap();
    
    // Parse CREATE TABLE
    let sql = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)";
    let mut lexer = Lexer::new(sql);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse_statement().unwrap();
    
    // Build plan
    let builder = PlanBuilder::new();
    let plan = builder.build(stmt).unwrap();
    
    // Execute CREATE TABLE
    catalog.create_table(&plan, &mut pager).unwrap();
    
    // Verify table exists
    assert!(catalog.get_table("users").is_some());
    let table = catalog.get_table("users").unwrap();
    assert_eq!(table.columns.len(), 2);
    assert_eq!(table.primary_key, Some(0));
    
    // Cleanup
    std::fs::remove_file("test_e2e_create.db").ok();
}

#[test]
fn test_end_to_end_select() {
    let mut pager = Pager::open("test_e2e_select.db").unwrap();
    let mut catalog = CatalogManager::new();
    catalog.load(&mut pager).unwrap();
    
    // Create table first
    let columns = vec![
        ColumnSpec {
            name: "id".to_string(),
            data_type: DataType::Integer,
            not_null: true,
            primary_key: true,
            unique: false,
            default: None,
        },
        ColumnSpec {
            name: "name".to_string(),
            data_type: DataType::Text,
            not_null: true,
            primary_key: false,
            unique: false,
            default: None,
        },
    ];
    
    let create_plan = LogicalPlan::CreateTable {
        table: "users".to_string(),
        columns,
    };
    
    catalog.create_table(&create_plan, &mut pager).unwrap();
    
    // Parse SELECT
    let sql = "SELECT * FROM users";
    let mut lexer = Lexer::new(sql);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse_statement().unwrap();
    
    // Build and execute SELECT
    let builder = PlanBuilder::new();
    let plan = builder.build(stmt).unwrap();
    let result = SelectExecutor::execute(plan, &catalog, &mut pager);
    
    assert!(result.is_ok());
    
    // Cleanup
    std::fs::remove_file("test_e2e_select.db").ok();
}

#[test]
fn test_order_by_sorting() {
    let mut rows = vec![
        vec![Value::Text("Charlie".to_string()), Value::Integer(30)],
        vec![Value::Text("Alice".to_string()), Value::Integer(25)],
        vec![Value::Text("Bob".to_string()), Value::Integer(35)],
    ];
    
    // Sort by name (column 0) ascending
    OrderByExecutor::sort_by_column(&mut rows, 0, OrderDirection::Asc).unwrap();
    
    assert_eq!(rows[0][0], Value::Text("Alice".to_string()));
    assert_eq!(rows[1][0], Value::Text("Bob".to_string()));
    assert_eq!(rows[2][0], Value::Text("Charlie".to_string()));
}

#[test]
fn test_order_by_sorting_numbers() {
    let mut rows = vec![
        vec![Value::Integer(30)],
        vec![Value::Integer(10)],
        vec![Value::Integer(20)],
    ];
    
    OrderByExecutor::sort_by_column(&mut rows, 0, OrderDirection::Desc).unwrap();
    
    assert_eq!(rows[0][0], Value::Integer(30));
    assert_eq!(rows[1][0], Value::Integer(20));
    assert_eq!(rows[2][0], Value::Integer(10));
}

#[test]
fn test_limit_only() {
    let rows = vec![
        vec![Value::Integer(1)],
        vec![Value::Integer(2)],
        vec![Value::Integer(3)],
        vec![Value::Integer(4)],
    ];
    
    let result = LimitExecutor::apply(rows, Some(2), None).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0], Value::Integer(1));
    assert_eq!(result[1][0], Value::Integer(2));
}

#[test]
fn test_offset_only() {
    let rows = vec![
        vec![Value::Integer(1)],
        vec![Value::Integer(2)],
        vec![Value::Integer(3)],
        vec![Value::Integer(4)],
    ];
    
    let result = LimitExecutor::apply(rows, None, Some(2)).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0], Value::Integer(3));
    assert_eq!(result[1][0], Value::Integer(4));
}

#[test]
fn test_limit_and_offset() {
    let rows = vec![
        vec![Value::Integer(1)],
        vec![Value::Integer(2)],
        vec![Value::Integer(3)],
        vec![Value::Integer(4)],
        vec![Value::Integer(5)],
    ];
    
    let result = LimitExecutor::apply(rows, Some(2), Some(1)).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0], Value::Integer(2));
    assert_eq!(result[1][0], Value::Integer(3));
}

#[test]
fn test_error_handling_table_not_found() {
    let mut pager = Pager::open("test_e2e_error.db").unwrap();
    let catalog = CatalogManager::new();
    
    // Try to delete from non-existent table
    let plan = LogicalPlan::Delete {
        table: "nonexistent".to_string(),
        filter: None,
    };
    
    let result = DeleteExecutor::execute(plan, &catalog, &mut pager);
    assert!(result.is_err());
    
    // Cleanup
    std::fs::remove_file("test_e2e_error.db").ok();
}

#[test]
fn test_insert_execution() {
    let mut pager = Pager::open("test_e2e_insert.db").unwrap();
    let mut catalog = CatalogManager::new();
    
    // Create table
    let create_plan = LogicalPlan::CreateTable {
        table: "test".to_string(),
        columns: vec![
            ColumnSpec {
                name: "id".to_string(),
                data_type: DataType::Integer,
                not_null: true,
                primary_key: true,
                unique: false,
                default: None,
            },
        ],
    };
    
    catalog.create_table(&create_plan, &mut pager).unwrap();
    
    // Insert data
    let insert_plan = LogicalPlan::Insert {
        table: "test".to_string(),
        columns: None,
        values: vec![vec![Expr::Literal(Literal::Integer(1))]],
    };
    
    let result = InsertExecutor::execute(insert_plan, &mut catalog, &mut pager);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows_affected, 1);
    
    // Cleanup
    std::fs::remove_file("test_e2e_insert.db").ok();
}

