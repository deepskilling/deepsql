/// DeepSQL CLI (placeholder for Phase 8)

use deepsql::{Engine, Result};
use deepsql::storage::record::{Record, Value};

fn main() -> Result<()> {
    println!("DeepSQL v0.1.0 - Storage Engine Demo");
    println!("=====================================\n");
    
    // Create a demo database
    let db_path = "demo.db";
    let mut engine = Engine::open(db_path)?;
    
    println!("✓ Database opened: {}", db_path);
    
    // Insert some test records
    println!("\nInserting test records...");
    for i in 1..=5 {
        let key = vec![i];
        let record = Record::new(
            key.clone(),
            vec![
                Value::Integer(i as i64),
                Value::Text(format!("Record {}", i)),
            ],
        );
        engine.insert(record)?;
        println!("  ✓ Inserted record with key: {:?}", key);
    }
    
    // Search for records
    println!("\nSearching for records...");
    for i in 1..=5 {
        let key = vec![i];
        match engine.search(&key) {
            Ok(record) => {
                println!("  ✓ Found key {:?}: {:?}", key, record.values);
            }
            Err(e) => {
                println!("  ✗ Key {:?} not found: {}", key, e);
            }
        }
    }
    
    // Scan all records
    println!("\nScanning all records...");
    let mut cursor = engine.scan()?;
    let mut count = 0;
    
    while cursor.is_valid() {
        if let Ok(record) = cursor.current(engine.pager_mut()) {
            println!("  {:?}: {:?}", record.key, record.values);
            count += 1;
        }
        
        if !cursor.next(engine.pager_mut())? {
            break;
        }
    }
    
    println!("\nTotal records scanned: {}", count);
    
    // Database stats
    let stats = engine.stats();
    println!("\nDatabase Statistics:");
    println!("  Pages: {}", stats.page_count);
    println!("  Page size: {} bytes", stats.page_size);
    println!("  Root page: {}", stats.root_page_id);
    
    // Flush to disk
    engine.flush()?;
    println!("\n✓ Changes flushed to disk");
    
    println!("\nPhase 1 complete! Storage engine is working. 🎉");
    
    Ok(())
}

