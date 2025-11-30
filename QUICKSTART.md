# DeepSQL Quick Start Guide

## 🚀 Get Started in 5 Minutes

### Prerequisites
- Rust 1.70+ (install from https://rustup.rs/)

### 1. Clone or Navigate to Project

```bash
cd /path/to/DEEPSQL
```

### 2. Build the Project

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_engine_basic_operations
```

### 4. Run the Demo

```bash
cargo run --release
```

**Expected output:**
```
DeepSQL v0.1.0 - Storage Engine Demo
=====================================

✓ Database opened: demo.db

Inserting test records...
  ✓ Inserted record with key: [1]
  ...

Searching for records...
  ✓ Found key [1]: [Integer(1), Text("Record 1")]
  ...

Scanning all records...
  [1]: [Integer(1), Text("Record 1")]
  ...

Total records scanned: 5

Database Statistics:
  Pages: 2
  Page size: 4096 bytes
  Root page: 2

✓ Changes flushed to disk

Phase 1 complete! Storage engine is working. 🎉
```

---

## 📖 Your First Database

Create a new file `examples/my_first_db.rs`:

```rust
use deepsql::{Engine, storage::record::{Record, Value}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open or create a database
    let mut db = Engine::open("test.db")?;
    
    // 2. Insert some records
    for i in 1..=10 {
        let key = vec![i];
        let record = Record::new(
            key,
            vec![
                Value::Integer(i as i64),
                Value::Text(format!("User {}", i)),
            ],
        );
        db.insert(record)?;
    }
    
    // 3. Search for a specific record
    let key = vec![5];
    let record = db.search(&key)?;
    println!("Found: {:?}", record.values);
    
    // 4. Scan all records
    let mut cursor = db.scan()?;
    let mut count = 0;
    
    while cursor.is_valid() {
        let record = cursor.current(db.pager_mut())?;
        println!("{:?}: {:?}", record.key, record.values);
        count += 1;
        
        if !cursor.next(db.pager_mut())? {
            break;
        }
    }
    
    println!("Total records: {}", count);
    
    // 5. Delete a record
    db.delete(&vec![5])?;
    
    // 6. Persist to disk
    db.flush()?;
    
    println!("Done!");
    Ok(())
}
```

Run it:
```bash
cargo run --example my_first_db
```

---

## 🧪 Interactive Rust Playground

Use `cargo` to experiment:

```bash
# Start Rust REPL (if you have it)
# Or create a new binary in examples/

# Create examples/playground.rs
cat > examples/playground.rs << 'EOF'
use deepsql::{Engine, storage::record::{Record, Value}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Engine::open("playground.db")?;
    
    // Your code here!
    
    Ok(())
}
EOF

# Run it
cargo run --example playground
```

---

## 📚 Common Operations

### Create/Open Database
```rust
let mut db = Engine::open("mydb.db")?;
```

### Insert Record
```rust
let record = Record::new(
    vec![1, 2, 3],                    // Key
    vec![
        Value::Integer(42),
        Value::Text("Hello".to_string()),
        Value::Real(3.14),
        Value::Blob(vec![0xDE, 0xAD]),
        Value::Null,
    ]
);
db.insert(record)?;
```

### Search by Key
```rust
let key = vec![1, 2, 3];
match db.search(&key) {
    Ok(record) => println!("Found: {:?}", record.values),
    Err(_) => println!("Not found"),
}
```

### Update Record (Insert with Existing Key)
```rust
let key = vec![1, 2, 3];
let updated = Record::new(key, vec![Value::Integer(100)]);
db.insert(updated)?;  // Overwrites existing
```

### Delete Record
```rust
db.delete(&vec![1, 2, 3])?;
```

### Scan All Records
```rust
let mut cursor = db.scan()?;

while cursor.is_valid() {
    let record = cursor.current(db.pager_mut())?;
    println!("{:?}: {:?}", record.key, record.values);
    
    if !cursor.next(db.pager_mut())? {
        break;
    }
}
```

### Get Database Stats
```rust
let stats = db.stats();
println!("Pages: {}", stats.page_count);
println!("Page size: {} bytes", stats.page_size);
println!("Root page: {}", stats.root_page_id);
```

### Flush to Disk
```rust
db.flush()?;
```

---

## 🔍 Debugging

### Enable Logging
Add to `Cargo.toml` (optional):
```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

### Check File Contents
```bash
# Show database file size
ls -lh mydb.db

# Hex dump first page (header)
hexdump -C mydb.db | head -20
```

### Run Tests with Backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Run Specific Module Tests
```bash
cargo test storage::pager
cargo test btree::insert
cargo test engine
```

---

## 🐛 Common Issues

### Problem: "No such file or directory"
**Solution**: Make sure you're in the DEEPSQL directory
```bash
cd /path/to/DEEPSQL
```

### Problem: "failed to fill whole buffer"
**Solution**: This was a known issue in development, already fixed. Make sure you have the latest code.

### Problem: Tests failing
**Solution**: Clean and rebuild
```bash
cargo clean
cargo test
```

### Problem: Database file locked
**Solution**: Make sure you close/drop the Engine properly
```rust
{
    let mut db = Engine::open("test.db")?;
    // use db...
} // db is dropped and flushed here
```

---

## 📊 Performance Tips

### 1. Use Release Mode
```bash
cargo run --release
cargo test --release
```

### 2. Batch Operations
```rust
// Insert many records before flushing
for i in 0..1000 {
    db.insert(record)?;
}
db.flush()?;  // Flush once at the end
```

### 3. Reuse Keys
```rust
// Keys are Vec<u8>, so reuse them
let key = vec![1, 2, 3];
db.insert(record.clone())?;
db.search(&key)?;
db.delete(&key)?;
```

---

## 📖 Next Steps

1. **Explore the code**: Start with `src/engine.rs` for high-level API
2. **Read the tests**: `tests/storage_tests.rs` has comprehensive examples
3. **Check the docs**: See `PHASE1_COMPLETE.md` for architecture details
4. **Build something**: Try creating a simple key-value store app
5. **Contribute**: Phase 2 needs WAL and transactions!

---

## 🆘 Need Help?

- **Documentation**: See `PHASE1_COMPLETE.md` and `IMPLEMENTATION_SUMMARY.md`
- **Examples**: Check `src/main.rs` for a working demo
- **Tests**: Look at `tests/storage_tests.rs` for usage patterns
- **Code**: Read inline comments in source files

---

## ✅ Verify Installation

Run this quick test:

```bash
cargo test --quiet && echo "✅ All tests passing!" || echo "❌ Tests failed"
```

Expected output:
```
✅ All tests passing!
```

---

**You're ready to use DeepSQL! 🎉**

Start with the demo:
```bash
cargo run --release
```

Or create your own database:
```rust
let mut db = Engine::open("myapp.db")?;
// Your code here!
```

Happy coding! 🦀

