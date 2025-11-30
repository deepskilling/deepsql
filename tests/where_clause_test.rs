/// WHERE Clause Resolution Test
use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_select_where() {
    let path = "/tmp/test_select_where.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("    SELECT WHERE TEST");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Setup
    engine.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)").unwrap();
    engine.execute("INSERT INTO users VALUES (1, 'Alice', 25)").unwrap();
    engine.execute("INSERT INTO users VALUES (2, 'Bob', 30)").unwrap();
    engine.execute("INSERT INTO users VALUES (3, 'Charlie', 35)").unwrap();
    
    println!("✅ Setup: 3 users inserted\n");
    
    // Test 1: SELECT without WHERE (baseline)
    println!("1️⃣  SELECT * FROM users (no WHERE):");
    let result1 = engine.execute("SELECT * FROM users").unwrap();
    println!("   Result: {:?}", result1);
    println!("   Rows: {}", result1.rows.len());
    assert_eq!(result1.rows.len(), 3);
    println!("   ✅ Works!\n");
    
    // Test 2: SELECT with simple WHERE
    println!("2️⃣  SELECT * FROM users WHERE id = 2:");
    let result2 = engine.execute("SELECT * FROM users WHERE id = 2");
    println!("   Result: {:?}", result2);
    
    match result2 {
        Ok(r) => {
            println!("   Rows: {}", r.rows.len());
            if r.rows.len() == 1 {
                println!("   ✅ WHERE clause WORKING!");
                println!("   Row: {:?}", r.rows[0]);
            } else if r.rows.len() == 0 {
                println!("   ⚠️  WHERE matched 0 rows (expected 1)");
            } else {
                println!("   ⚠️  WHERE matched {} rows (expected 1)", r.rows.len());
            }
        }
        Err(e) => {
            println!("   ❌ WHERE clause FAILED: {:?}", e);
        }
    }
    
    println!("\n═══════════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

