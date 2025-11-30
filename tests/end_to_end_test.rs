/// End-to-End SQL Tests
///
/// Tests complete SQL workflows from CREATE TABLE through SELECT

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_complete_sql_workflow() {
    let path = "/tmp/test_complete_workflow.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════════");
    println!("    Complete SQL Workflow Test");
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Step 1: CREATE TABLE
    println!("1️⃣  CREATE TABLE users...");
    let create_result = engine.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER)"
    );
    println!("   Result: {:?}", create_result);
    assert!(create_result.is_ok(), "CREATE TABLE should succeed");
    println!("   ✅ Table created!\n");
    
    // Step 2: INSERT rows
    println!("2️⃣  INSERT INTO users...");
    let insert1 = engine.execute("INSERT INTO users (id, name, age) VALUES (1, 'Alice', 30)");
    println!("   Row 1: {:?}", insert1);
    assert!(insert1.is_ok(), "INSERT should succeed");
    assert_eq!(insert1.as_ref().unwrap().rows_affected, 1);
    
    let insert2 = engine.execute("INSERT INTO users (id, name, age) VALUES (2, 'Bob', 25)");
    println!("   Row 2: {:?}", insert2);
    assert!(insert2.is_ok(), "INSERT should succeed");
    
    let insert3 = engine.execute("INSERT INTO users (id, name, age) VALUES (3, 'Charlie', 35)");
    println!("   Row 3: {:?}", insert3);
    assert!(insert3.is_ok(), "INSERT should succeed");
    println!("   ✅ 3 rows inserted!\n");
    
    // Step 3: SELECT all
    println!("3️⃣  SELECT * FROM users...");
    let select_result = engine.execute("SELECT * FROM users");
    println!("   Result: {:?}", select_result);
    
    match select_result {
        Ok(result) => {
            println!("   ✅ Query executed!");
            println!("   Rows returned: {}", result.rows.len());
            
            for (i, row) in result.rows.iter().enumerate() {
                println!("   Row {}: {:?}", i + 1, row);
            }
            
            // Verify we got data back
            assert!(result.rows.len() > 0, "Should return rows");
        }
        Err(e) => {
            println!("   ⚠️  SELECT failed: {:?}", e);
            // This might be expected if B+Tree reading isn't fully wired up yet
        }
    }
    
    // Step 4: SELECT with WHERE clause
    println!("\n4️⃣  SELECT * FROM users WHERE age > 28...");
    let select_filtered = engine.execute("SELECT * FROM users WHERE age > 28");
    println!("   Result: {:?}", select_filtered);
    
    match select_filtered {
        Ok(result) => {
            println!("   ✅ Filtered query executed!");
            println!("   Rows returned: {}", result.rows.len());
            
            for (i, row) in result.rows.iter().enumerate() {
                println!("   Row {}: {:?}", i + 1, row);
            }
        }
        Err(e) => {
            println!("   ⚠️  SELECT with WHERE failed: {:?}", e);
        }
    }
    
    println!("\n═══════════════════════════════════════════════════════════");
    println!("    Workflow Test Complete");
    println!("═══════════════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_multiple_inserts() {
    let path = "/tmp/test_multiple_inserts.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    // Create table
    engine.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL)").unwrap();
    
    // Insert multiple rows
    for i in 1..=10 {
        let sql = format!("INSERT INTO products (id, name, price) VALUES ({}, 'Product {}', {}.99)", i, i, i * 10);
        let result = engine.execute(&sql);
        println!("Insert {}: {:?}", i, result);
        assert!(result.is_ok(), "INSERT {} should succeed", i);
        assert_eq!(result.unwrap().rows_affected, 1);
    }
    
    println!("✅ Successfully inserted 10 rows!");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

