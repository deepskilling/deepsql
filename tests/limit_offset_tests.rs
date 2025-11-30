/// Tests for LIMIT and OFFSET functionality

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use deepsql::types::Value;
use std::fs;

#[test]
fn test_limit_basic() {
    let path = "/tmp/test_limit_basic.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 LIMIT BASIC TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
    engine.execute("INSERT INTO items VALUES (1, 'First')").unwrap();
    engine.execute("INSERT INTO items VALUES (2, 'Second')").unwrap();
    engine.execute("INSERT INTO items VALUES (3, 'Third')").unwrap();
    engine.execute("INSERT INTO items VALUES (4, 'Fourth')").unwrap();
    engine.execute("INSERT INTO items VALUES (5, 'Fifth')").unwrap();
    println!("✅ Setup: 5 items inserted");
    
    // Test LIMIT
    println!("\nExecuting: SELECT * FROM items LIMIT 3");
    let result = engine.execute("SELECT * FROM items LIMIT 3").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 3, "Should return exactly 3 rows");
    println!("✅ LIMIT 3 returned {} rows", result.rows.len());
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_offset_basic() {
    let path = "/tmp/test_offset_basic.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 OFFSET BASIC TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE numbers (id INTEGER PRIMARY KEY, val INTEGER)").unwrap();
    engine.execute("INSERT INTO numbers VALUES (1, 10)").unwrap();
    engine.execute("INSERT INTO numbers VALUES (2, 20)").unwrap();
    engine.execute("INSERT INTO numbers VALUES (3, 30)").unwrap();
    engine.execute("INSERT INTO numbers VALUES (4, 40)").unwrap();
    println!("✅ Setup: 4 numbers inserted");
    
    // Test OFFSET
    println!("\nExecuting: SELECT * FROM numbers LIMIT 2 OFFSET 2");
    let result = engine.execute("SELECT * FROM numbers LIMIT 2 OFFSET 2").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 2, "Should return 2 rows after skipping 2");
    
    // Verify we got rows 3 and 4
    if let Value::Integer(id) = &result.rows[0][0] {
        assert_eq!(*id, 3, "First result should be id=3");
        println!("✅ First row: id = {}", id);
    }
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_limit_with_order_by() {
    let path = "/tmp/test_limit_order.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 LIMIT WITH ORDER BY TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE scores (id INTEGER PRIMARY KEY, score INTEGER)").unwrap();
    engine.execute("INSERT INTO scores VALUES (1, 85)").unwrap();
    engine.execute("INSERT INTO scores VALUES (2, 95)").unwrap();
    engine.execute("INSERT INTO scores VALUES (3, 75)").unwrap();
    engine.execute("INSERT INTO scores VALUES (4, 90)").unwrap();
    println!("✅ Setup: 4 scores inserted (85, 95, 75, 90)");
    
    // Test LIMIT with ORDER BY
    println!("\nExecuting: SELECT * FROM scores ORDER BY score DESC LIMIT 2");
    let result = engine.execute("SELECT * FROM scores ORDER BY score DESC LIMIT 2").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 2, "Should return top 2 scores");
    
    // Verify we got the top 2: 95 and 90
    let scores: Vec<i64> = result.rows.iter()
        .map(|row| {
            if let Value::Integer(score) = &row[1] {
                *score
            } else {
                0
            }
        })
        .collect();
    
    assert_eq!(scores, vec![95, 90], "Should be top 2 scores in DESC order");
    println!("✅ Top 2 scores: {:?}", scores);
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_limit_zero() {
    let path = "/tmp/test_limit_zero.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 LIMIT 0 TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE data (id INTEGER PRIMARY KEY)").unwrap();
    engine.execute("INSERT INTO data VALUES (1)").unwrap();
    engine.execute("INSERT INTO data VALUES (2)").unwrap();
    println!("✅ Setup: 2 rows inserted");
    
    // Test LIMIT 0
    println!("\nExecuting: SELECT * FROM data LIMIT 0");
    let result = engine.execute("SELECT * FROM data LIMIT 0").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 0, "LIMIT 0 should return no rows");
    println!("✅ LIMIT 0 returned 0 rows");
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_limit_exceeds_rows() {
    let path = "/tmp/test_limit_exceeds.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 LIMIT EXCEEDS ROWS TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and populate
    engine.execute("CREATE TABLE small (id INTEGER PRIMARY KEY)").unwrap();
    engine.execute("INSERT INTO small VALUES (1)").unwrap();
    engine.execute("INSERT INTO small VALUES (2)").unwrap();
    println!("✅ Setup: 2 rows inserted");
    
    // Test LIMIT > row count
    println!("\nExecuting: SELECT * FROM small LIMIT 10");
    let result = engine.execute("SELECT * FROM small LIMIT 10").unwrap();
    println!("Result: {:?}", result.rows);
    
    assert_eq!(result.rows.len(), 2, "Should return all 2 rows (not 10)");
    println!("✅ LIMIT 10 returned {} rows (all available)", result.rows.len());
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

