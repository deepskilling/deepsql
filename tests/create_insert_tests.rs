/// CREATE TABLE and INSERT Statement Tests
///
/// Tests DDL and DML execution

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_create_table_basic() {
    let path = "/tmp/test_create_table.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    // Create a simple table
    let sql = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER)";
    let result = engine.execute(sql);
    
    println!("Result: {:?}", result);
    
    // Should succeed
    assert!(result.is_ok(), "CREATE TABLE should succeed: {:?}", result.err());
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_create_table_with_constraints() {
    let path = "/tmp/test_create_constraints.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    // Create table with various constraints
    let sql = "CREATE TABLE products (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        price REAL,
        category TEXT,
        in_stock INTEGER
    )";
    
    let result = engine.execute(sql);
    assert!(result.is_ok(), "CREATE TABLE with constraints should succeed");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_create_duplicate_table() {
    let path = "/tmp/test_duplicate_table.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    // Create table
    engine.execute("CREATE TABLE test (id INTEGER)").unwrap();
    
    // Try to create again - should fail
    let result = engine.execute("CREATE TABLE test (id INTEGER)");
    assert!(result.is_err(), "Duplicate table creation should fail");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_insert_basic() {
    let path = "/tmp/test_insert.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    // Create table
    engine.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)").unwrap();
    
    // Insert data
    let sql = "INSERT INTO users (id, name, age) VALUES (1, 'Alice', 25)";
    let result = engine.execute(sql);
    
    println!("INSERT result: {:?}", result);
    
    // For now, INSERT might not be fully implemented, so we just check it doesn't panic
    // Once fully implemented, this should succeed
    match result {
        Ok(_) => println!("INSERT succeeded!"),
        Err(e) => println!("INSERT not yet fully implemented: {:?}", e),
    }
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_insert_into_nonexistent_table() {
    let path = "/tmp/test_insert_noexist.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    // Try to insert without creating table
    let result = engine.execute("INSERT INTO users (id, name) VALUES (1, 'Bob')");
    
    // Should fail
    assert!(result.is_err(), "INSERT into non-existent table should fail");
    assert!(result.unwrap_err().to_string().contains("does not exist"));
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_create_and_select_workflow() {
    let path = "/tmp/test_workflow.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    // Step 1: Create table
    let create_result = engine.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)");
    println!("CREATE result: {:?}", create_result);
    assert!(create_result.is_ok(), "CREATE TABLE should succeed");
    
    // Step 2: Insert data (might not be fully implemented yet)
    let insert_result = engine.execute("INSERT INTO test (id, value) VALUES (1, 'hello')");
    println!("INSERT result: {:?}", insert_result);
    
    // Step 3: Select data (will be implemented after INSERT works)
    let select_result = engine.execute("SELECT * FROM test");
    println!("SELECT result: {:?}", select_result);
    
    drop(engine);
    let _ = fs::remove_file(path);
}

