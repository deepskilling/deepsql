/// Index Manager
/// 
/// Manages index lifecycle and operations

use crate::catalog::schema::IndexSchema;
use crate::error::{Error, Result};
use crate::index::index_btree::{IndexBTree, IndexKeyBuilder};
use crate::storage::pager::Pager;
use crate::types::Value;
use std::collections::HashMap;

/// Index manager - manages all indexes
pub struct IndexManager {
    /// Loaded indexes (index_name -> IndexBTree)
    indexes: HashMap<String, IndexBTree>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new() -> Self {
        IndexManager {
            indexes: HashMap::new(),
        }
    }
    
    /// Load an index
    pub fn load_index(&mut self, schema: &IndexSchema, pager: &mut Pager) -> Result<()> {
        let index = IndexBTree::new(pager, schema.root_page, schema.unique)?;
        self.indexes.insert(schema.name.clone(), index);
        Ok(())
    }
    
    /// Get an index by name
    pub fn get_index(&mut self, name: &str) -> Option<&mut IndexBTree> {
        self.indexes.get_mut(name)
    }
    
    /// Insert into index
    pub fn insert_into_index(
        &mut self,
        index_name: &str,
        values: &[Value],
        row_id: u64,
        pager: &mut Pager,
    ) -> Result<()> {
        let index = self.indexes.get_mut(index_name)
            .ok_or_else(|| Error::Internal(format!("Index '{}' not found", index_name)))?;
        
        let key = IndexKeyBuilder::build_key(values);
        index.insert(pager, &key, row_id)?;
        
        Ok(())
    }
    
    /// Delete from index
    pub fn delete_from_index(
        &mut self,
        index_name: &str,
        values: &[Value],
        pager: &mut Pager,
    ) -> Result<()> {
        let index = self.indexes.get_mut(index_name)
            .ok_or_else(|| Error::Internal(format!("Index '{}' not found", index_name)))?;
        
        let key = IndexKeyBuilder::build_key(values);
        index.delete(pager, &key)?;
        
        Ok(())
    }
    
    /// Search index for a value
    pub fn search_index(
        &mut self,
        index_name: &str,
        values: &[Value],
        pager: &mut Pager,
    ) -> Result<Option<u64>> {
        let index = self.indexes.get_mut(index_name)
            .ok_or_else(|| Error::Internal(format!("Index '{}' not found", index_name)))?;
        
        let key = IndexKeyBuilder::build_key(values);
        index.search(pager, &key)
    }
    
    /// Update index entry (delete old + insert new)
    pub fn update_index(
        &mut self,
        index_name: &str,
        old_values: &[Value],
        new_values: &[Value],
        row_id: u64,
        pager: &mut Pager,
    ) -> Result<()> {
        self.delete_from_index(index_name, old_values, pager)?;
        self.insert_into_index(index_name, new_values, row_id, pager)?;
        Ok(())
    }
    
    /// Get all indexes on a table
    pub fn get_table_indexes(&self, _table_name: &str) -> Vec<String> {
        // TODO: Filter indexes by table
        self.indexes.keys().cloned().collect()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::schema::ColumnType;
    
    #[test]
    fn test_index_manager_creation() {
        let manager = IndexManager::new();
        assert_eq!(manager.indexes.len(), 0);
    }
    
    #[test]
    fn test_load_index() {
        let mut manager = IndexManager::new();
        let mut pager = Pager::open("test_index_mgr.db").unwrap();
        
        let mut schema = IndexSchema::new(
            "idx_users_email".to_string(),
            "users".to_string(),
            2,
        );
        schema.add_column("email".to_string());
        schema.unique = true;
        
        manager.load_index(&schema, &mut pager).unwrap();
        
        assert!(manager.get_index("idx_users_email").is_some());
        
        // Cleanup
        std::fs::remove_file("test_index_mgr.db").ok();
    }
}

