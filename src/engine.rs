/// Database Engine - Main API facade
/// 
/// Provides a high-level interface for interacting with DeepSQL

use crate::error::{Error, Result};
use crate::storage::{Pager, btree::BTree, record::Record};
use crate::wal::{Wal, checkpoint, recover};
use crate::locking::LockManager;
use crate::transaction::TransactionContext;
use std::path::Path;

/// Database engine instance
pub struct Engine {
    /// Page manager
    pager: Pager,
    
    /// Main table B+Tree
    btree: BTree,
    
    /// Write-Ahead Log
    wal: Wal,
    
    /// Lock manager
    lock_manager: LockManager,
    
    /// Transaction context
    tx_context: TransactionContext,
    
    /// Database path
    path: std::path::PathBuf,
}

impl Engine {
    /// Open or create a database
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Acquire shared lock for opening
        let mut lock_manager = LockManager::new(&path)?;
        lock_manager.lock_shared()?;
        
        let mut pager = Pager::open(&path)?;
        let page_size = pager.page_size() as u32;
        
        // Open WAL and perform recovery if needed
        let mut wal = Wal::open(&path, page_size)?;
        let _recovered = recover(&mut pager, &mut wal)?;
        
        // Recovery logging can be added in production
        
        // Check if database has a root page
        let root_page_id = pager.root_page();
        
        let btree = if root_page_id == 0 {
            // New database, create root
            BTree::new(&mut pager)?
        } else {
            // Existing database, open root
            BTree::open(root_page_id)?
        };
        
        Ok(Engine { 
            pager, 
            btree, 
            wal, 
            lock_manager,
            tx_context: TransactionContext::new(),
            path,
        })
    }
    
    /// Begin a transaction
    pub fn begin_transaction(&mut self) -> Result<()> {
        // Upgrade to exclusive lock
        self.lock_manager.lock_exclusive()?;
        
        // Enable transaction tracking
        self.tx_context.begin()?;
        self.pager.begin_transaction_mode();
        self.wal.begin_transaction()
    }
    
    /// Commit the current transaction
    pub fn commit_transaction(&mut self) -> Result<()> {
        if !self.wal.in_transaction() {
            return Err(Error::Internal("No active transaction".to_string()));
        }
        
        if !self.tx_context.is_active() {
            return Err(Error::Internal("No active transaction context".to_string()));
        }
        
        // Step 1: Collect modified pages  
        let modified_pages: Vec<_> = self.pager.modified_pages().values().cloned().collect();
        
        // Step 2: Write modified pages to WAL
        for page in &modified_pages {
            self.wal.write_page(page)?;
        }
        
        // Step 3: Commit WAL
        let db_size = self.pager.page_count();
        self.wal.commit_transaction(db_size)?;
        
        // Step 4: Write modified pages to disk (now durable via WAL)
        self.pager.commit_transaction_pages()?;
        
        // Step 5: Clear transaction state
        self.tx_context.commit();
        self.pager.end_transaction_mode();
        
        // Check if checkpoint is needed
        if self.wal.needs_checkpoint() {
            checkpoint(&mut self.pager, &mut self.wal)?;
        }
        
        Ok(())
    }
    
    /// Rollback the current transaction
    pub fn rollback_transaction(&mut self) -> Result<()> {
        if !self.tx_context.is_active() {
            return Err(Error::Internal("No active transaction".to_string()));
        }
        
        // Restore original pages from pager's shadow copies
        self.pager.rollback_transaction_pages()?;
        
        // Clear transaction state
        self.tx_context.clear();
        self.pager.end_transaction_mode();
        
        // Rollback WAL
        self.wal.rollback_transaction()?;
        
        Ok(())
    }
    
    /// Insert a record (auto-transaction)
    pub fn insert(&mut self, record: Record) -> Result<()> {
        let auto_transaction = !self.wal.in_transaction();
        
        if auto_transaction {
            self.begin_transaction()?;
        }
        
        self.btree.insert(&mut self.pager, record)?;
        
        // Write modified pages to WAL
        // Note: In a full implementation, we'd track which pages were modified
        // For now, we'll checkpoint after each auto-transaction
        
        if auto_transaction {
            self.commit_transaction()?;
        }
        
        Ok(())
    }
    
    /// Search for a record by key
    pub fn search(&mut self, key: &[u8]) -> Result<Record> {
        self.btree.search(&mut self.pager, key)
    }
    
    /// Delete a record by key (auto-transaction)
    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        let auto_transaction = !self.wal.in_transaction();
        
        if auto_transaction {
            self.begin_transaction()?;
        }
        
        self.btree.delete(&mut self.pager, key)?;
        
        if auto_transaction {
            self.commit_transaction()?;
        }
        
        Ok(())
    }
    
    /// Create a cursor for scanning records
    pub fn scan(&mut self) -> Result<crate::storage::btree::Cursor> {
        self.btree.cursor(&mut self.pager)
    }
    
    /// Get mutable reference to pager (for cursor operations)
    pub fn pager_mut(&mut self) -> &mut Pager {
        &mut self.pager
    }
    
    /// Get mutable reference to transaction context (internal use)
    #[allow(dead_code)]
    pub(crate) fn tx_context_mut(&mut self) -> &mut TransactionContext {
        &mut self.tx_context
    }
    
    /// Flush all changes to disk
    pub fn flush(&mut self) -> Result<()> {
        // Checkpoint WAL to database
        checkpoint(&mut self.pager, &mut self.wal)?;
        self.pager.flush()
    }
    
    /// Perform a checkpoint (copy WAL to main database)
    pub fn checkpoint(&mut self) -> Result<usize> {
        checkpoint(&mut self.pager, &mut self.wal)
    }
    
    /// Get database statistics
    pub fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            page_count: self.pager.page_count(),
            page_size: self.pager.page_size(),
            root_page_id: self.btree.root_page_id(),
            wal_frames: self.wal.frame_count(),
            in_transaction: self.wal.in_transaction(),
        }
    }
    
    /// Get database path
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Flush on drop
        let _ = self.flush();
        let _ = self.lock_manager.unlock();
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Total number of pages
    pub page_count: u32,
    
    /// Page size in bytes
    pub page_size: usize,
    
    /// Root page ID
    pub root_page_id: u32,
    
    /// WAL frame count
    pub wal_frames: u32,
    
    /// In transaction?
    pub in_transaction: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::record::Value;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_engine_basic_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut engine = Engine::open(temp_file.path()).unwrap();
        
        // Insert
        let key = vec![1, 2, 3];
        let record = Record::new(key.clone(), vec![Value::Integer(42)]);
        engine.insert(record).unwrap();
        
        // Search
        let found = engine.search(&key).unwrap();
        assert_eq!(found.key, key);
        assert_eq!(found.values[0], Value::Integer(42));
        
        // Delete
        engine.delete(&key).unwrap();
        assert!(engine.search(&key).is_err());
    }
    
    #[test]
    fn test_engine_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        
        // Create and insert
        {
            let mut engine = Engine::open(&path).unwrap();
            let key = vec![1];
            let record = Record::new(key, vec![Value::Integer(100)]);
            engine.insert(record).unwrap();
            engine.flush().unwrap();
        }
        
        // Reopen and verify
        {
            let mut engine = Engine::open(&path).unwrap();
            let found = engine.search(&[1]).unwrap();
            assert_eq!(found.values[0], Value::Integer(100));
        }
    }
}

