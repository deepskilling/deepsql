/// Basic CREATE INDEX tests

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_create_index_basic() {
    let path = "/tmp/test_create_index_basic.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 CREATE INDEX BASIC TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create table
    engine.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)").unwrap();
    println!("✅ Table created");
    
    // Create index
    println!("\nExecuting: CREATE INDEX idx_age ON users (age)");
    let result = engine.execute("CREATE INDEX idx_age ON users (age)").unwrap();
    println!("Result: {:?}", result);
    println!("✅ Index created");
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_create_unique_index() {
    let path = "/tmp/test_create_unique_index.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n🔍 CREATE UNIQUE INDEX TEST");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create table
    engine.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, sku TEXT)").unwrap();
    println!("✅ Table created");
    
    // Create unique index
    println!("\nExecuting: CREATE UNIQUE INDEX idx_sku ON products (sku)");
    let result = engine.execute("CREATE UNIQUE INDEX idx_sku ON products (sku)").unwrap();
    println!("Result: {:?}", result);
    println!("✅ Unique index created");
    
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Cleanup
    let _ = fs::remove_file(path);
}

