/// Debug UPDATE WHERE execution

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_update_where_simple() {
    let path = "/tmp/test_update_where_simple.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 UPDATE WHERE DEBUG TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create and insert
    engine.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val INTEGER)").unwrap();
    engine.execute("INSERT INTO test VALUES (1, 100)").unwrap();
    engine.execute("INSERT INTO test VALUES (2, 200)").unwrap();
    engine.execute("INSERT INTO test VALUES (3, 300)").unwrap();
    println!("✅ Setup complete");
    
    // Check initial state
    let result = engine.execute("SELECT * FROM test").unwrap();
    println!("\nBefore UPDATE:");
    for row in &result.rows {
        println!("  {:?}", row);
    }
    
    // Try UPDATE with WHERE
    println!("\nExecuting: UPDATE test SET val = 999 WHERE id = 2");
    let result = engine.execute("UPDATE test SET val = 999 WHERE id = 2").unwrap();
    println!("Result: rows_affected = {}", result.rows_affected);
    
    // Check final state
    let result = engine.execute("SELECT * FROM test").unwrap();
    println!("\nAfter UPDATE:");
    for row in &result.rows {
        println!("  {:?}", row);
    }
    
    // Cleanup
    let _ = fs::remove_file(path);
}

