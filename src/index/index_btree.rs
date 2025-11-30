/// Index B+Tree
/// 
/// Secondary index implementation using B+Tree structure
/// 
/// Note: This is a foundational structure for Phase 6.
/// Full integration with the B+Tree layer requires additional refactoring
/// of the Record and B+Tree APIs to support raw key-value storage.

use crate::error::Result;
use crate::storage::pager::Pager;
use crate::types::Value;

/// Index B+Tree - maps indexed values to row IDs
pub struct IndexBTree {
    /// Root page of the index
    root_page: u32,
    
    /// Whether this is a unique index
    unique: bool,
    
    /// In-memory index map (simplified implementation)
    /// Maps key bytes to row IDs
    index_map: std::collections::HashMap<Vec<u8>, Vec<u64>>,
}

impl IndexBTree {
    /// Create a new index B+Tree
    pub fn new(_pager: &mut Pager, root_page: u32, unique: bool) -> Result<Self> {
        Ok(IndexBTree { 
            root_page,
            unique,
            index_map: std::collections::HashMap::new(),
        })
    }
    
    /// Insert an entry into the index
    /// Key: indexed column value(s), Value: row ID
    pub fn insert(&mut self, _pager: &mut Pager, key: &[u8], row_id: u64) -> Result<()> {
        // Simplified: Use in-memory HashMap
        // TODO: Persist to actual B+Tree pages
        
        let key_vec = key.to_vec();
        
        if self.unique {
            // Check for duplicate in unique index
            if self.index_map.contains_key(&key_vec) {
                return Err(crate::error::Error::Internal(
                    "Unique constraint violation".into()
                ));
            }
            self.index_map.insert(key_vec, vec![row_id]);
        } else {
            // Non-unique: append to existing list
            self.index_map.entry(key_vec)
                .or_insert_with(Vec::new)
                .push(row_id);
        }
        
        Ok(())
    }
    
    /// Delete an entry from the index
    pub fn delete(&mut self, _pager: &mut Pager, key: &[u8]) -> Result<()> {
        // Simplified: Remove from HashMap
        // TODO: Delete from actual B+Tree
        self.index_map.remove(key);
        Ok(())
    }
    
    /// Search for a key in the index
    /// Returns the row ID if found
    pub fn search(&mut self, _pager: &mut Pager, key: &[u8]) -> Result<Option<u64>> {
        // Simplified: Lookup in HashMap
        // TODO: Search actual B+Tree
        Ok(self.index_map.get(key).and_then(|ids| ids.first().copied()))
    }
    
    /// Scan the index for keys in a range
    pub fn scan_range(
        &mut self,
        _pager: &mut Pager,
        _start_key: Option<&[u8]>,
        _end_key: Option<&[u8]>,
    ) -> Result<Vec<u64>> {
        // Simplified: Return all row IDs
        // TODO: Range scan on actual B+Tree
        let all_ids: Vec<u64> = self.index_map.values()
            .flat_map(|ids| ids.iter().copied())
            .collect();
        Ok(all_ids)
    }
    
    /// Check if this is a unique index
    pub fn is_unique(&self) -> bool {
        self.unique
    }
}

/// Index key builder - converts values to index keys
pub struct IndexKeyBuilder;

impl IndexKeyBuilder {
    /// Build an index key from values
    pub fn build_key(values: &[Value]) -> Vec<u8> {
        let mut key = Vec::new();
        
        for value in values {
            match value {
                Value::Null => {
                    key.push(0); // NULL marker
                }
                Value::Integer(i) => {
                    key.push(1); // Integer marker
                    key.extend_from_slice(&i.to_be_bytes());
                }
                Value::Real(r) => {
                    key.push(2); // Real marker
                    key.extend_from_slice(&r.to_be_bytes());
                }
                Value::Text(s) => {
                    key.push(3); // Text marker
                    key.extend_from_slice(s.as_bytes());
                    key.push(0); // String terminator
                }
                Value::Blob(b) => {
                    key.push(4); // Blob marker
                    key.extend_from_slice(&(b.len() as u32).to_be_bytes());
                    key.extend_from_slice(b);
                }
            }
        }
        
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_index_key_builder() {
        let values = vec![
            Value::Integer(42),
            Value::Text("test".to_string()),
        ];
        
        let key = IndexKeyBuilder::build_key(&values);
        
        // Should have markers and encoded values
        assert!(!key.is_empty());
        assert_eq!(key[0], 1); // Integer marker
    }
    
    #[test]
    fn test_index_btree_creation() {
        let mut pager = Pager::open("test_index.db").unwrap();
        let index = IndexBTree::new(&mut pager, 2, true).unwrap();
        
        assert!(index.is_unique());
        
        // Cleanup
        std::fs::remove_file("test_index.db").ok();
    }
}

