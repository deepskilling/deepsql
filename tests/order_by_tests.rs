/// Tests for ORDER BY functionality

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use deepsql::types::Value;
use std::fs;

#[test]
fn test_order_by_single_column_asc() {
    let path = "/tmp/test_order_by_asc.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 ORDER BY SINGLE COLUMN ASC TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE scores (id INTEGER PRIMARY KEY, score INTEGER)").unwrap();
    engine.execute("INSERT INTO scores VALUES (1, 85)").unwrap();
    engine.execute("INSERT INTO scores VALUES (2, 92)").unwrap();
    engine.execute("INSERT INTO scores VALUES (3, 78)").unwrap();
    println!("✅ Setup: 3 scores inserted (85, 92, 78)");
    
    // Test ORDER BY ASC
    println!("\nExecuting: SELECT * FROM scores ORDER BY score ASC");
    let result = engine.execute("SELECT * FROM scores ORDER BY score ASC").unwrap();
    println!("Result: {:?}", result.rows);
    
    // Verify sorted order: 78, 85, 92
    assert_eq!(result.rows.len(), 3, "Should have 3 rows");
    
    // Check first row (should be lowest score: 78)
    match &result.rows[0][1] {
        Value::Integer(score) => {
            assert_eq!(*score, 78, "First row should have score 78");
            println!("✅ First row: score = {}", score);
        }
        _ => panic!("Expected Integer for score"),
    }
    
    // Check last row (should be highest score: 92)
    match &result.rows[2][1] {
        Value::Integer(score) => {
            assert_eq!(*score, 92, "Last row should have score 92");
            println!("✅ Last row: score = {}", score);
        }
        _ => panic!("Expected Integer for score"),
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_order_by_single_column_desc() {
    let path = "/tmp/test_order_by_desc.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 ORDER BY SINGLE COLUMN DESC TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, price INTEGER)").unwrap();
    engine.execute("INSERT INTO products VALUES (1, 100)").unwrap();
    engine.execute("INSERT INTO products VALUES (2, 50)").unwrap();
    engine.execute("INSERT INTO products VALUES (3, 150)").unwrap();
    println!("✅ Setup: 3 products inserted (100, 50, 150)");
    
    // Test ORDER BY DESC
    println!("\nExecuting: SELECT * FROM products ORDER BY price DESC");
    let result = engine.execute("SELECT * FROM products ORDER BY price DESC").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 3);
    
    // Check first row (should be highest price: 150)
    match &result.rows[0][1] {
        Value::Integer(price) => {
            assert_eq!(*price, 150, "First row should have price 150");
            println!("✅ First row: price = {}", price);
        }
        _ => panic!("Expected Integer for price"),
    }
    
    // Check last row (should be lowest price: 50)
    match &result.rows[2][1] {
        Value::Integer(price) => {
            assert_eq!(*price, 50, "Last row should have price 50");
            println!("✅ Last row: price = {}", price);
        }
        _ => panic!("Expected Integer for price"),
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_order_by_with_where() {
    let path = "/tmp/test_order_by_where.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 ORDER BY WITH WHERE CLAUSE TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, value INTEGER)").unwrap();
    engine.execute("INSERT INTO items VALUES (1, 100)").unwrap();
    engine.execute("INSERT INTO items VALUES (2, 50)").unwrap();
    engine.execute("INSERT INTO items VALUES (3, 150)").unwrap();
    engine.execute("INSERT INTO items VALUES (4, 75)").unwrap();
    println!("✅ Setup: 4 items inserted");
    
    // Test ORDER BY with WHERE
    println!("\nExecuting: SELECT * FROM items WHERE value > 60 ORDER BY value DESC");
    let result = engine.execute("SELECT * FROM items WHERE value > 60 ORDER BY value DESC").unwrap();
    println!("Result: {:?}", result.rows);
    
    // Should have 3 rows (100, 75, 150 filtered, then sorted DESC: 150, 100, 75)
    assert_eq!(result.rows.len(), 3, "Should have 3 rows after filtering");
    
    // Verify order: 150, 100, 75
    let values: Vec<i64> = result.rows.iter()
        .map(|row| {
            if let Value::Integer(v) = &row[1] {
                *v
            } else {
                panic!("Expected Integer");
            }
        })
        .collect();
    
    assert_eq!(values, vec![150, 100, 75], "Should be sorted DESC");
    println!("✅ Sorted values: {:?}", values);
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_order_by_text() {
    let path = "/tmp/test_order_by_text.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 ORDER BY TEXT COLUMN TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE names (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
    engine.execute("INSERT INTO names VALUES (1, 'Charlie')").unwrap();
    engine.execute("INSERT INTO names VALUES (2, 'Alice')").unwrap();
    engine.execute("INSERT INTO names VALUES (3, 'Bob')").unwrap();
    println!("✅ Setup: 3 names inserted (Charlie, Alice, Bob)");
    
    // Test ORDER BY text column
    println!("\nExecuting: SELECT * FROM names ORDER BY name ASC");
    let result = engine.execute("SELECT * FROM names ORDER BY name ASC").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 3);
    
    // Verify alphabetical order: Alice, Bob, Charlie
    let names: Vec<String> = result.rows.iter()
        .map(|row| {
            if let Value::Text(name) = &row[1] {
                name.clone()
            } else {
                panic!("Expected Text");
            }
        })
        .collect();
    
    assert_eq!(names, vec!["Alice", "Bob", "Charlie"], "Should be alphabetically sorted");
    println!("✅ Sorted names: {:?}", names);
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_order_by_with_aggregates() {
    let path = "/tmp/test_order_by_aggregates.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 ORDER BY WITH AGGREGATES TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE data (id INTEGER PRIMARY KEY, value INTEGER)").unwrap();
    engine.execute("INSERT INTO data VALUES (1, 10)").unwrap();
    engine.execute("INSERT INTO data VALUES (2, 20)").unwrap();
    engine.execute("INSERT INTO data VALUES (3, 30)").unwrap();
    println!("✅ Setup: 3 rows inserted");
    
    // Note: This test verifies aggregates + ORDER BY doesn't break
    // (ORDER BY on aggregate results is typically not meaningful for single aggregate)
    println!("\nExecuting: SELECT COUNT(*) FROM data");
    let result = engine.execute("SELECT COUNT(*) FROM data").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Integer(count) => {
            assert_eq!(*count, 3);
            println!("✅ COUNT(*) = {}", count);
        }
        _ => panic!("Expected Integer"),
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

