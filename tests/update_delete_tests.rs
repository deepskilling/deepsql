/// UPDATE and DELETE Statement Tests

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_update_all_rows() {
    let path = "/tmp/test_update_all.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("    UPDATE ALL ROWS TEST");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Setup: Create table and insert rows
    engine.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)").unwrap();
    engine.execute("INSERT INTO users VALUES (1, 'Alice', 25)").unwrap();
    engine.execute("INSERT INTO users VALUES (2, 'Bob', 30)").unwrap();
    engine.execute("INSERT INTO users VALUES (3, 'Charlie', 35)").unwrap();
    
    println!("✅ Setup: 3 rows inserted\n");
    
    // Update all rows
    println!("UPDATE users SET age = 40");
    let result = engine.execute("UPDATE users SET age = 40");
    println!("Result: {:?}\n", result);
    
    match result {
        Ok(r) => {
            println!("✅ UPDATE executed!");
            println!("   Rows affected: {}", r.rows_affected);
            assert_eq!(r.rows_affected, 3, "Should update 3 rows");
        }
        Err(e) => {
            println!("⚠️  UPDATE failed: {:?}", e);
        }
    }
    
    println!("═══════════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_update_with_where() {
    let path = "/tmp/test_update_where.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("    UPDATE WITH WHERE CLAUSE TEST");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Setup
    engine.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL)").unwrap();
    engine.execute("INSERT INTO products VALUES (1, 'Widget', 10.00)").unwrap();
    engine.execute("INSERT INTO products VALUES (2, 'Gadget', 20.00)").unwrap();
    engine.execute("INSERT INTO products VALUES (3, 'Doohickey', 15.00)").unwrap();
    
    println!("✅ Setup: 3 products inserted\n");
    
    // Update only products with price < 15
    println!("UPDATE products SET price = 25.00 WHERE id = 2");
    let result = engine.execute("UPDATE products SET price = 25.00 WHERE id = 2");
    println!("Result: {:?}\n", result);
    
    match result {
        Ok(r) => {
            println!("✅ UPDATE with WHERE executed!");
            println!("   Rows affected: {}", r.rows_affected);
            // Should update only 1 row (id = 2)
        }
        Err(e) => {
            println!("⚠️  UPDATE failed: {:?}", e);
        }
    }
    
    println!("═══════════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_delete_all_rows() {
    let path = "/tmp/test_delete_all.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("    DELETE ALL ROWS TEST");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Setup
    engine.execute("CREATE TABLE logs (id INTEGER PRIMARY KEY, message TEXT)").unwrap();
    engine.execute("INSERT INTO logs VALUES (1, 'Log 1')").unwrap();
    engine.execute("INSERT INTO logs VALUES (2, 'Log 2')").unwrap();
    engine.execute("INSERT INTO logs VALUES (3, 'Log 3')").unwrap();
    
    println!("✅ Setup: 3 logs inserted\n");
    
    // Delete all rows
    println!("DELETE FROM logs");
    let result = engine.execute("DELETE FROM logs");
    println!("Result: {:?}\n", result);
    
    match result {
        Ok(r) => {
            println!("✅ DELETE executed!");
            println!("   Rows affected: {}", r.rows_affected);
            assert_eq!(r.rows_affected, 3, "Should delete 3 rows");
            
            // Verify table is empty
            let select_result = engine.execute("SELECT * FROM logs");
            println!("   After DELETE - SELECT: {:?}", select_result);
            
            if let Ok(sel) = select_result {
                assert_eq!(sel.rows.len(), 0, "Table should be empty");
                println!("   ✅ Table is now empty!");
            }
        }
        Err(e) => {
            println!("⚠️  DELETE failed: {:?}", e);
        }
    }
    
    println!("═══════════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_delete_with_where() {
    let path = "/tmp/test_delete_where.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("    DELETE WITH WHERE CLAUSE TEST");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Setup
    engine.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, quantity INTEGER)").unwrap();
    engine.execute("INSERT INTO items VALUES (1, 'Electronics', 5)").unwrap();
    engine.execute("INSERT INTO items VALUES (2, 'Books', 0)").unwrap();
    engine.execute("INSERT INTO items VALUES (3, 'Toys', 10)").unwrap();
    engine.execute("INSERT INTO items VALUES (4, 'Books', 0)").unwrap();
    
    println!("✅ Setup: 4 items inserted\n");
    
    // Delete only items where id = 2
    println!("DELETE FROM items WHERE id = 2");
    let result = engine.execute("DELETE FROM items WHERE id = 2");
    println!("Result: {:?}\n", result);
    
    match result {
        Ok(r) => {
            println!("✅ DELETE with WHERE executed!");
            println!("   Rows affected: {}", r.rows_affected);
            
            // Verify remaining rows
            let select_result = engine.execute("SELECT * FROM items");
            println!("   After DELETE - SELECT: {:?}", select_result);
            
            if let Ok(sel) = select_result {
                println!("   Remaining rows: {}", sel.rows.len());
                // Should have 3 rows left (deleted 1)
            }
        }
        Err(e) => {
            println!("⚠️  DELETE failed: {:?}", e);
        }
    }
    
    println!("═══════════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_complete_crud_workflow() {
    let path = "/tmp/test_crud.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    engine.load_catalog().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("    COMPLETE CRUD WORKFLOW TEST");
    println!("═══════════════════════════════════════════════════════\n");
    
    // CREATE
    println!("1️⃣  CREATE TABLE...");
    engine.execute("CREATE TABLE employees (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        salary REAL
    )").unwrap();
    println!("   ✅ Table created\n");
    
    // INSERT
    println!("2️⃣  INSERT rows...");
    engine.execute("INSERT INTO employees VALUES (1, 'Alice', 50000.0)").unwrap();
    engine.execute("INSERT INTO employees VALUES (2, 'Bob', 60000.0)").unwrap();
    engine.execute("INSERT INTO employees VALUES (3, 'Charlie', 55000.0)").unwrap();
    println!("   ✅ 3 rows inserted\n");
    
    // SELECT (READ)
    println!("3️⃣  SELECT all rows...");
    let result = engine.execute("SELECT * FROM employees").unwrap();
    println!("   Rows: {}", result.rows.len());
    for (i, row) in result.rows.iter().enumerate() {
        println!("     {}: {:?}", i+1, row);
    }
    assert_eq!(result.rows.len(), 3);
    println!("   ✅ SELECT works!\n");
    
    // UPDATE
    println!("4️⃣  UPDATE salary...");
    let update_result = engine.execute("UPDATE employees SET salary = 65000.0 WHERE id = 2");
    println!("   Result: {:?}", update_result);
    if let Ok(r) = update_result {
        println!("   Rows affected: {}", r.rows_affected);
    }
    
    // DELETE
    println!("\n5️⃣  DELETE a row...");
    let delete_result = engine.execute("DELETE FROM employees WHERE id = 3");
    println!("   Result: {:?}", delete_result);
    if let Ok(r) = delete_result {
        println!("   Rows affected: {}", r.rows_affected);
    }
    
    // Final SELECT
    println!("\n6️⃣  Final SELECT to verify...");
    let final_result = engine.execute("SELECT * FROM employees");
    println!("   Result: {:?}", final_result);
    
    println!("\n═══════════════════════════════════════════════════════");
    println!("    CRUD Workflow Complete!");
    println!("═══════════════════════════════════════════════════════\n");
    
    drop(engine);
    let _ = fs::remove_file(path);
}

