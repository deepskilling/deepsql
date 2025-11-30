/// Debug test to understand SELECT behavior
use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn debug_select_columns() {
    let path = "/tmp/debug_select.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n════════════════════════════════════════════════");
    println!("DEBUG: Testing SELECT Column Reading");
    println!("════════════════════════════════════════════════\n");
    
    // Create simple table with 3 columns
    println!("1. Creating table with 3 columns...");
    engine.execute("CREATE TABLE test (
        id INTEGER PRIMARY KEY,
        name TEXT,
        age INTEGER
    )").unwrap();
    println!("   ✅ Table created\n");
    
    // Insert one row with explicit values
    println!("2. Inserting row: (1, 'Alice', 30)");
    let insert_result = engine.execute("INSERT INTO test VALUES (1, 'Alice', 30)");
    println!("   Result: {:?}", insert_result);
    assert!(insert_result.is_ok());
    println!("   ✅ Row inserted\n");
    
    // Select all columns
    println!("3. SELECT * FROM test");
    let select_result = engine.execute("SELECT * FROM test");
    println!("   Result: {:?}\n", select_result);
    
    if let Ok(result) = select_result {
        println!("   Number of rows: {}", result.rows.len());
        
        for (i, row) in result.rows.iter().enumerate() {
            println!("   Row {}: {} values", i, row.len());
            for (j, value) in row.iter().enumerate() {
                println!("     Column {}: {:?}", j, value);
            }
        }
        
        // Check expectations
        assert_eq!(result.rows.len(), 1, "Should have 1 row");
        
        if result.rows[0].len() == 1 {
            println!("\n   ⚠️  BUG CONFIRMED: Only {} column returned (expected 3)", result.rows[0].len());
            println!("   This means Column opcodes aren't being generated or executed");
        } else if result.rows[0].len() == 3 {
            println!("\n   ✅ SUCCESS: All 3 columns returned!");
        } else {
            println!("\n   ⚠️  UNEXPECTED: {} columns returned", result.rows[0].len());
        }
    }
    
    println!("\n════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

