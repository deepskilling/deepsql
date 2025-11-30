/// Integration tests for Phase 2: WAL + ACID Transactions

use deepsql::{Engine, storage::record::{Record, Value}};
use tempfile::NamedTempFile;

#[test]
fn test_transaction_commit() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Begin transaction
    engine.begin_transaction().unwrap();
    
    // Insert a record
    let key = vec![1];
    let record = Record::new(key.clone(), vec![Value::Integer(42)]);
    engine.insert(record).unwrap();
    
    // Commit
    engine.commit_transaction().unwrap();
    
    // Verify record exists
    let found = engine.search(&key).unwrap();
    assert_eq!(found.values[0], Value::Integer(42));
}

#[test]
fn test_transaction_rollback() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Begin transaction
    engine.begin_transaction().unwrap();
    
    // Insert a record
    let key = vec![1];
    let record = Record::new(key.clone(), vec![Value::Integer(42)]);
    engine.insert(record).unwrap();
    
    // Rollback
    engine.rollback_transaction().unwrap();
    
    // Verify record does not exist
    assert!(engine.search(&key).is_err());
}

#[test]
fn test_multiple_transactions() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Transaction 1
    engine.begin_transaction().unwrap();
    let record1 = Record::new(vec![1], vec![Value::Integer(100)]);
    engine.insert(record1).unwrap();
    engine.commit_transaction().unwrap();
    
    // Transaction 2
    engine.begin_transaction().unwrap();
    let record2 = Record::new(vec![2], vec![Value::Integer(200)]);
    engine.insert(record2).unwrap();
    engine.commit_transaction().unwrap();
    
    // Verify both records
    let found1 = engine.search(&[1]).unwrap();
    let found2 = engine.search(&[2]).unwrap();
    assert_eq!(found1.values[0], Value::Integer(100));
    assert_eq!(found2.values[0], Value::Integer(200));
}

#[test]
fn test_crash_recovery() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_path_buf();
    
    // Write a transaction
    {
        let mut engine = Engine::open(&temp_path).unwrap();
        engine.begin_transaction().unwrap();
        
        let record = Record::new(vec![1], vec![Value::Integer(999)]);
        engine.insert(record).unwrap();
        
        engine.commit_transaction().unwrap();
        // Simulate crash - don't checkpoint
        drop(engine);
    }
    
    // Reopen and verify recovery
    {
        let mut engine = Engine::open(&temp_path).unwrap();
        let found = engine.search(&[1]).unwrap();
        assert_eq!(found.values[0], Value::Integer(999));
    }
}

#[test]
fn test_checkpoint() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Write multiple transactions
    for i in 0..10 {
        engine.begin_transaction().unwrap();
        let record = Record::new(vec![i], vec![Value::Integer(i as i64)]);
        engine.insert(record).unwrap();
        engine.commit_transaction().unwrap();
    }
    
    // Perform checkpoint
    let pages_written = engine.checkpoint().unwrap();
    assert!(pages_written > 0);
    
    // Verify all records still exist
    for i in 0..10 {
        let found = engine.search(&[i]).unwrap();
        assert_eq!(found.values[0], Value::Integer(i as i64));
    }
}

#[test]
fn test_auto_transaction() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert without explicit transaction (auto-transaction)
    let record = Record::new(vec![1], vec![Value::Integer(42)]);
    engine.insert(record).unwrap();
    
    // Verify it was committed
    let found = engine.search(&[1]).unwrap();
    assert_eq!(found.values[0], Value::Integer(42));
}

#[test]
fn test_transaction_isolation() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert record outside transaction
    engine.begin_transaction().unwrap();
    let record1 = Record::new(vec![1], vec![Value::Integer(1)]);
    engine.insert(record1).unwrap();
    engine.commit_transaction().unwrap();
    
    // Start new transaction and insert another record
    engine.begin_transaction().unwrap();
    let record2 = Record::new(vec![2], vec![Value::Integer(2)]);
    engine.insert(record2).unwrap();
    
    // Rollback second transaction
    engine.rollback_transaction().unwrap();
    
    // Verify first record exists, second doesn't
    assert!(engine.search(&[1]).is_ok());
    assert!(engine.search(&[2]).is_err());
}

#[test]
fn test_durability_after_flush() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_path_buf();
    
    // Write and flush
    {
        let mut engine = Engine::open(&temp_path).unwrap();
        let record = Record::new(vec![1], vec![Value::Integer(42)]);
        engine.insert(record).unwrap();
        engine.flush().unwrap();
    }
    
    // Reopen and verify
    {
        let mut engine = Engine::open(&temp_path).unwrap();
        let found = engine.search(&[1]).unwrap();
        assert_eq!(found.values[0], Value::Integer(42));
    }
}

#[test]
fn test_wal_stats() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Write some transactions
    for i in 0..5 {
        engine.begin_transaction().unwrap();
        let record = Record::new(vec![i], vec![Value::Integer(i as i64)]);
        engine.insert(record).unwrap();
        engine.commit_transaction().unwrap();
    }
    
    // Check stats
    let stats = engine.stats();
    assert!(stats.wal_frames > 0);
    assert!(!stats.in_transaction);
}

#[test]
fn test_large_transaction() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Begin transaction
    engine.begin_transaction().unwrap();
    
    // Insert many records
    for i in 0..100 {
        let record = Record::new(vec![i], vec![Value::Integer(i as i64 * 10)]);
        engine.insert(record).unwrap();
    }
    
    // Commit
    engine.commit_transaction().unwrap();
    
    // Verify all records
    for i in 0..100 {
        let found = engine.search(&[i]).unwrap();
        assert_eq!(found.values[0], Value::Integer(i as i64 * 10));
    }
}

#[test]
fn test_update_in_transaction() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert initial value
    engine.begin_transaction().unwrap();
    let record1 = Record::new(vec![1], vec![Value::Integer(100)]);
    engine.insert(record1).unwrap();
    engine.commit_transaction().unwrap();
    
    // Update in new transaction
    engine.begin_transaction().unwrap();
    let record2 = Record::new(vec![1], vec![Value::Integer(200)]);
    engine.insert(record2).unwrap();
    engine.commit_transaction().unwrap();
    
    // Verify updated value
    let found = engine.search(&[1]).unwrap();
    assert_eq!(found.values[0], Value::Integer(200));
}

#[test]
fn test_delete_in_transaction() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert
    engine.begin_transaction().unwrap();
    let record = Record::new(vec![1], vec![Value::Integer(42)]);
    engine.insert(record).unwrap();
    engine.commit_transaction().unwrap();
    
    // Delete in transaction
    engine.begin_transaction().unwrap();
    engine.delete(&[1]).unwrap();
    engine.commit_transaction().unwrap();
    
    // Verify deleted
    assert!(engine.search(&[1]).is_err());
}

#[test]
fn test_mixed_operations_in_transaction() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Setup initial data
    for i in 0..5 {
        let record = Record::new(vec![i], vec![Value::Integer(i as i64)]);
        engine.insert(record).unwrap();
    }
    
    // Transaction with mixed operations
    engine.begin_transaction().unwrap();
    
    // Update some records
    let record = Record::new(vec![1], vec![Value::Integer(100)]);
    engine.insert(record).unwrap();
    
    // Delete some records
    engine.delete(&[2]).unwrap();
    
    // Insert new records
    let new_record = Record::new(vec![10], vec![Value::Integer(1000)]);
    engine.insert(new_record).unwrap();
    
    engine.commit_transaction().unwrap();
    
    // Verify final state
    assert_eq!(engine.search(&[1]).unwrap().values[0], Value::Integer(100));
    assert!(engine.search(&[2]).is_err());
    assert_eq!(engine.search(&[10]).unwrap().values[0], Value::Integer(1000));
}

