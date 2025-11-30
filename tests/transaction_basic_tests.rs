/// Basic transaction tests

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_begin_commit() {
    let path = "/tmp/test_begin_commit.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 BEGIN/COMMIT TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Begin transaction
    println!("\nExecuting: BEGIN");
    engine.execute("BEGIN").unwrap();
    println!("✅ Transaction started");
    
    // Commit transaction
    println!("\nExecuting: COMMIT");
    engine.execute("COMMIT").unwrap();
    println!("✅ Transaction committed");
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_begin_rollback() {
    let path = "/tmp/test_begin_rollback.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 BEGIN/ROLLBACK TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Begin transaction
    println!("\nExecuting: BEGIN");
    engine.execute("BEGIN").unwrap();
    println!("✅ Transaction started");
    
    // Rollback transaction
    println!("\nExecuting: ROLLBACK");
    engine.execute("ROLLBACK").unwrap();
    println!("✅ Transaction rolled back");
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_nested_transaction_error() {
    let path = "/tmp/test_nested_tx.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 NESTED TRANSACTION ERROR TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Begin transaction
    engine.execute("BEGIN").unwrap();
    println!("✅ First BEGIN succeeded");
    
    // Try nested BEGIN (should fail)
    println!("\nTrying nested BEGIN...");
    let result = engine.execute("BEGIN");
    assert!(result.is_err(), "Nested BEGIN should fail");
    println!("✅ Nested BEGIN correctly rejected");
    
    // Cleanup
    engine.execute("COMMIT").unwrap();
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

