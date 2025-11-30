/// Debug test for WHERE clause implementation

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_where_debug() {
    let path = "/tmp/test_where_debug.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 WHERE CLAUSE DEBUG TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create table
    println!("\n1️⃣  CREATE TABLE");
    engine.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value INTEGER)").unwrap();
    println!("   ✅ Table created");
    
    // Insert test data
    println!("\n2️⃣  INSERT DATA");
    engine.execute("INSERT INTO test VALUES (1, 100)").unwrap();
    engine.execute("INSERT INTO test VALUES (2, 200)").unwrap();
    engine.execute("INSERT INTO test VALUES (3, 300)").unwrap();
    println!("   ✅ 3 rows inserted");
    
    // Test 1: SELECT without WHERE
    println!("\n3️⃣  SELECT * FROM test (no WHERE):");
    let result = engine.execute("SELECT * FROM test").unwrap();
    println!("   Result: {:?}", result);
    println!("   Rows: {}", result.rows.len());
    assert_eq!(result.rows.len(), 3);
    println!("   ✅ Correct!");
    
    // Test 2: SELECT with WHERE id = 2
    println!("\n4️⃣  SELECT * FROM test WHERE id = 2:");
    let result = engine.execute("SELECT * FROM test WHERE id = 2").unwrap();
    println!("   Result: {:?}", result);
    println!("   Rows: {}", result.rows.len());
    
    if result.rows.len() == 1 {
        println!("   ✅ WHERE clause works! Found 1 row");
        assert_eq!(result.rows[0][0], deepsql::types::Value::Integer(2));
        assert_eq!(result.rows[0][1], deepsql::types::Value::Integer(200));
    } else {
        println!("   ❌ WHERE clause NOT working - found {} rows (expected 1)", result.rows.len());
        for (i, row) in result.rows.iter().enumerate() {
            println!("      Row {}: {:?}", i, row);
        }
    }
    
    println!("\n═══════════════════════════════════════════════════════════");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

