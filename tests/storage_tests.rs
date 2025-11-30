/// Integration tests for Phase 1: Storage Engine Foundation

use deepsql::Engine;
use deepsql::storage::record::{Record, Value, Varint};
use tempfile::NamedTempFile;

#[test]
fn test_varint_encoding() {
    // Test various integer sizes
    let test_cases = vec![
        0u64,
        1,
        127,
        128,
        255,
        256,
        16383,
        16384,
        u64::MAX,
    ];
    
    for value in test_cases {
        let encoded = Varint::encode(value);
        let (decoded, consumed) = Varint::decode(&encoded).unwrap();
        assert_eq!(value, decoded);
        assert_eq!(encoded.len(), consumed);
    }
}

#[test]
fn test_varint_signed_encoding() {
    let test_cases = vec![
        0i64,
        1,
        -1,
        127,
        -127,
        128,
        -128,
        i64::MAX,
        i64::MIN,
    ];
    
    for value in test_cases {
        let encoded = Varint::encode_signed(value);
        let (decoded, consumed) = Varint::decode_signed(&encoded).unwrap();
        assert_eq!(value, decoded);
        assert_eq!(encoded.len(), consumed);
    }
}

#[test]
fn test_value_types() {
    let test_values = vec![
        Value::Null,
        Value::Integer(42),
        Value::Integer(-1000),
        Value::Real(3.14159),
        Value::Real(-2.71828),
        Value::Text("Hello, World!".to_string()),
        Value::Text("".to_string()),
        Value::Blob(vec![1, 2, 3, 4, 5]),
        Value::Blob(vec![]),
    ];
    
    for value in test_values {
        let serialized = value.serialize();
        let (deserialized, consumed) = Value::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
        assert_eq!(serialized.len(), consumed);
    }
}

#[test]
fn test_record_serialization() {
    let record = Record::new(
        vec![1, 2, 3, 4],
        vec![
            Value::Integer(42),
            Value::Text("test".to_string()),
            Value::Real(3.14),
            Value::Blob(vec![5, 6, 7]),
            Value::Null,
        ],
    );
    
    let serialized = record.serialize();
    let deserialized = Record::deserialize(&serialized).unwrap();
    
    assert_eq!(record.key, deserialized.key);
    assert_eq!(record.values.len(), deserialized.values.len());
    
    for (v1, v2) in record.values.iter().zip(deserialized.values.iter()) {
        assert_eq!(v1, v2);
    }
}

#[test]
fn test_engine_create_database() {
    let temp_file = NamedTempFile::new().unwrap();
    let engine = Engine::open(temp_file.path()).unwrap();
    
    let stats = engine.stats();
    assert!(stats.page_count >= 1);
    assert_eq!(stats.page_size, 4096);
}

#[test]
fn test_engine_insert_and_search() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert a record
    let key = vec![1, 2, 3];
    let record = Record::new(
        key.clone(),
        vec![
            Value::Integer(42),
            Value::Text("test".to_string()),
        ],
    );
    
    engine.insert(record).unwrap();
    
    // Search for it
    let found = engine.search(&key).unwrap();
    assert_eq!(found.key, key);
    assert_eq!(found.values[0], Value::Integer(42));
    assert_eq!(found.values[1], Value::Text("test".to_string()));
}

#[test]
fn test_engine_insert_multiple_records() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert 100 records
    for i in 0..100 {
        let key = vec![(i / 256) as u8, (i % 256) as u8];
        let record = Record::new(
            key,
            vec![
                Value::Integer(i as i64),
                Value::Text(format!("Record {}", i)),
            ],
        );
        engine.insert(record).unwrap();
    }
    
    // Verify all records
    for i in 0..100 {
        let key = vec![(i / 256) as u8, (i % 256) as u8];
        let found = engine.search(&key).unwrap();
        assert_eq!(found.values[0], Value::Integer(i as i64));
        assert_eq!(found.values[1], Value::Text(format!("Record {}", i)));
    }
}

#[test]
fn test_engine_update_record() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    let key = vec![1, 2, 3];
    
    // Insert initial record
    let record1 = Record::new(key.clone(), vec![Value::Integer(42)]);
    engine.insert(record1).unwrap();
    
    // Update with new value
    let record2 = Record::new(key.clone(), vec![Value::Integer(100)]);
    engine.insert(record2).unwrap();
    
    // Verify updated value
    let found = engine.search(&key).unwrap();
    assert_eq!(found.values[0], Value::Integer(100));
}

#[test]
fn test_engine_delete_record() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    let key = vec![1, 2, 3];
    let record = Record::new(key.clone(), vec![Value::Integer(42)]);
    
    // Insert
    engine.insert(record).unwrap();
    assert!(engine.search(&key).is_ok());
    
    // Delete
    engine.delete(&key).unwrap();
    assert!(engine.search(&key).is_err());
}

#[test]
fn test_engine_delete_nonexistent() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    let result = engine.delete(&[1, 2, 3]);
    assert!(result.is_err());
}

#[test]
fn test_engine_cursor_scan() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert records
    for i in 0..10 {
        let key = vec![i];
        let record = Record::new(key, vec![Value::Integer(i as i64)]);
        engine.insert(record).unwrap();
    }
    
    // Scan all records
    let mut cursor = engine.scan().unwrap();
    let mut count = 0;
    let mut last_key = None;
    
    while cursor.is_valid() {
        let record = cursor.current(engine.pager_mut()).unwrap();
        
        // Verify records are in order
        if let Some(last) = last_key {
            assert!(record.key > last);
        }
        last_key = Some(record.key.clone());
        
        count += 1;
        
        if !cursor.next(engine.pager_mut()).unwrap() {
            break;
        }
    }
    
    assert_eq!(count, 10);
}

#[test]
fn test_engine_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    
    // Create database and insert records
    {
        let mut engine = Engine::open(&path).unwrap();
        
        for i in 0..10 {
            let key = vec![i];
            let record = Record::new(key, vec![Value::Integer(i as i64 * 10)]);
            engine.insert(record).unwrap();
        }
        
        engine.flush().unwrap();
    }
    
    // Reopen and verify all records
    {
        let mut engine = Engine::open(&path).unwrap();
        
        for i in 0..10 {
            let key = vec![i];
            let found = engine.search(&key).unwrap();
            assert_eq!(found.values[0], Value::Integer(i as i64 * 10));
        }
    }
}

#[test]
fn test_engine_mixed_operations() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Insert some records
    for i in 0..5 {
        let key = vec![i];
        let record = Record::new(key, vec![Value::Integer(i as i64)]);
        engine.insert(record).unwrap();
    }
    
    // Delete some records
    engine.delete(&[1]).unwrap();
    engine.delete(&[3]).unwrap();
    
    // Insert more records
    for i in 5..8 {
        let key = vec![i];
        let record = Record::new(key, vec![Value::Integer(i as i64)]);
        engine.insert(record).unwrap();
    }
    
    // Verify final state
    let existing = vec![0, 2, 4, 5, 6, 7];
    for i in existing {
        let key = vec![i];
        let found = engine.search(&key);
        assert!(found.is_ok(), "Key {:?} should exist", key);
    }
    
    let deleted = vec![1, 3];
    for i in deleted {
        let key = vec![i];
        let found = engine.search(&key);
        assert!(found.is_err(), "Key {:?} should not exist", key);
    }
}

#[test]
fn test_large_records() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    // Create a large text value
    let large_text = "x".repeat(1000);
    
    let key = vec![1];
    let record = Record::new(
        key.clone(),
        vec![
            Value::Text(large_text.clone()),
            Value::Blob(vec![42; 500]),
        ],
    );
    
    engine.insert(record).unwrap();
    
    let found = engine.search(&key).unwrap();
    assert_eq!(found.values[0], Value::Text(large_text));
    assert_eq!(found.values[1], Value::Blob(vec![42; 500]));
}

#[test]
fn test_empty_values() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut engine = Engine::open(temp_file.path()).unwrap();
    
    let key = vec![1];
    let record = Record::new(
        key.clone(),
        vec![
            Value::Text("".to_string()),
            Value::Blob(vec![]),
            Value::Null,
        ],
    );
    
    engine.insert(record).unwrap();
    
    let found = engine.search(&key).unwrap();
    assert_eq!(found.values[0], Value::Text("".to_string()));
    assert_eq!(found.values[1], Value::Blob(vec![]));
    assert_eq!(found.values[2], Value::Null);
}

