/// Tests for aggregate functions (COUNT, SUM, AVG, MIN, MAX)

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use deepsql::types::Value;
use std::fs;

#[test]
fn test_count_star() {
    let path = "/tmp/test_count_star.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 COUNT(*) TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
    engine.execute("INSERT INTO users VALUES (1, 'Alice')").unwrap();
    engine.execute("INSERT INTO users VALUES (2, 'Bob')").unwrap();
    engine.execute("INSERT INTO users VALUES (3, 'Charlie')").unwrap();
    println!("✅ Setup: 3 users inserted");
    
    // Test COUNT(*)
    println!("\nExecuting: SELECT COUNT(*) FROM users");
    let result = engine.execute("SELECT COUNT(*) FROM users").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 1, "Should have 1 result row");
    assert_eq!(result.rows[0].len(), 1, "Should have 1 column");
    
    match &result.rows[0][0] {
        Value::Integer(count) => {
            assert_eq!(*count, 3, "COUNT(*) should be 3");
            println!("✅ COUNT(*) = {} (correct!)", count);
        }
        _ => panic!("Expected Integer result for COUNT(*), got {:?}", result.rows[0][0]),
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_count_column() {
    let path = "/tmp/test_count_column.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 COUNT(column) TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
    engine.execute("INSERT INTO products VALUES (1, 'Apple')").unwrap();
    engine.execute("INSERT INTO products VALUES (2, 'Banana')").unwrap();
    println!("✅ Setup: 2 products inserted");
    
    // Test COUNT(column)
    println!("\nExecuting: SELECT COUNT(name) FROM products");
    let result = engine.execute("SELECT COUNT(name) FROM products").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Integer(count) => {
            assert_eq!(*count, 2, "COUNT(name) should be 2");
            println!("✅ COUNT(name) = {} (correct!)", count);
        }
        _ => panic!("Expected Integer for COUNT"),
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_sum() {
    let path = "/tmp/test_sum.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 SUM TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, amount INTEGER)").unwrap();
    engine.execute("INSERT INTO orders VALUES (1, 100)").unwrap();
    engine.execute("INSERT INTO orders VALUES (2, 200)").unwrap();
    engine.execute("INSERT INTO orders VALUES (3, 300)").unwrap();
    println!("✅ Setup: 3 orders inserted (100, 200, 300)");
    
    // Test SUM
    println!("\nExecuting: SELECT SUM(amount) FROM orders");
    let result = engine.execute("SELECT SUM(amount) FROM orders").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Integer(sum) => {
            assert_eq!(*sum, 600, "SUM(amount) should be 600");
            println!("✅ SUM(amount) = {} (correct!)", sum);
        }
        _ => panic!("Expected Integer for SUM"),
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_min_max() {
    let path = "/tmp/test_min_max.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 MIN/MAX TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE scores (id INTEGER PRIMARY KEY, score INTEGER)").unwrap();
    engine.execute("INSERT INTO scores VALUES (1, 85)").unwrap();
    engine.execute("INSERT INTO scores VALUES (2, 92)").unwrap();
    engine.execute("INSERT INTO scores VALUES (3, 78)").unwrap();
    println!("✅ Setup: 3 scores inserted (85, 92, 78)");
    
    // Test MIN
    println!("\nExecuting: SELECT MIN(score) FROM scores");
    let result = engine.execute("SELECT MIN(score) FROM scores").unwrap();
    println!("MIN Result: {:?}", result.rows);
    
    match &result.rows[0][0] {
        Value::Integer(min) => {
            assert_eq!(*min, 78, "MIN(score) should be 78");
            println!("✅ MIN(score) = {}", min);
        }
        _ => panic!("Expected Integer for MIN"),
    }
    
    // Test MAX
    println!("\nExecuting: SELECT MAX(score) FROM scores");
    let result = engine.execute("SELECT MAX(score) FROM scores").unwrap();
    println!("MAX Result: {:?}", result.rows);
    
    match &result.rows[0][0] {
        Value::Integer(max) => {
            assert_eq!(*max, 92, "MAX(score) should be 92");
            println!("✅ MAX(score) = {}", max);
        }
        _ => panic!("Expected Integer for MAX"),
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

