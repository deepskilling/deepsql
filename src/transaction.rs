/// Transaction context for tracking page modifications
/// 
/// Implements shadow paging: saves original pages before modification
/// to enable proper rollback and isolation

use crate::error::{Error, Result};
use crate::storage::{Page, PageId};
use std::collections::HashMap;

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// No active transaction
    None,
    
    /// Transaction in progress
    Active,
    
    /// Transaction committed
    Committed,
    
    /// Transaction rolled back
    RolledBack,
}

/// Transaction context tracks all changes during a transaction
pub struct TransactionContext {
    /// Original page data (before modification) for rollback
    /// Maps page_id -> original_page_data
    original_pages: HashMap<PageId, Vec<u8>>,
    
    /// Modified pages during this transaction
    /// Maps page_id -> modified_page
    modified_pages: HashMap<PageId, Page>,
    
    /// Transaction state
    state: TransactionState,
}

impl TransactionContext {
    /// Create a new transaction context
    pub fn new() -> Self {
        TransactionContext {
            original_pages: HashMap::new(),
            modified_pages: HashMap::new(),
            state: TransactionState::None,
        }
    }
    
    /// Begin a transaction
    pub fn begin(&mut self) -> Result<()> {
        if self.state == TransactionState::Active {
            return Err(Error::Internal("Transaction already active".to_string()));
        }
        
        self.original_pages.clear();
        self.modified_pages.clear();
        self.state = TransactionState::Active;
        Ok(())
    }
    
    /// Save original page data before first modification
    pub fn save_original(&mut self, page_id: PageId, original_data: Vec<u8>) {
        // Only save the first version (for rollback)
        if !self.original_pages.contains_key(&page_id) {
            self.original_pages.insert(page_id, original_data);
        }
    }
    
    /// Track a modified page
    pub fn track_modified_page(&mut self, page: Page) {
        self.modified_pages.insert(page.id, page);
    }
    
    /// Get a modified page (if it exists in transaction)
    pub fn get_modified_page(&self, page_id: PageId) -> Option<&Page> {
        self.modified_pages.get(&page_id)
    }
    
    /// Get all modified pages
    pub fn modified_pages(&self) -> &HashMap<PageId, Page> {
        &self.modified_pages
    }
    
    /// Get original page data for rollback
    pub fn get_original_data(&self, page_id: PageId) -> Option<&Vec<u8>> {
        self.original_pages.get(&page_id)
    }
    
    /// Commit the transaction
    pub fn commit(&mut self) {
        self.state = TransactionState::Committed;
        self.original_pages.clear();
        self.modified_pages.clear();
    }
    
    /// Rollback the transaction, returning original pages to restore
    pub fn rollback(&mut self) -> HashMap<PageId, Vec<u8>> {
        self.state = TransactionState::RolledBack;
        self.modified_pages.clear();
        std::mem::take(&mut self.original_pages)
    }
    
    /// Check if transaction is active
    pub fn is_active(&self) -> bool {
        self.state == TransactionState::Active
    }
    
    /// Get transaction state
    pub fn state(&self) -> TransactionState {
        self.state
    }
    
    /// Clear transaction state
    pub fn clear(&mut self) {
        self.original_pages.clear();
        self.modified_pages.clear();
        self.state = TransactionState::None;
    }
}

impl Default for TransactionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_lifecycle() {
        let mut ctx = TransactionContext::new();
        assert!(!ctx.is_active());
        
        ctx.begin().unwrap();
        assert!(ctx.is_active());
        
        ctx.commit();
        assert!(!ctx.is_active());
    }
    
    #[test]
    fn test_save_original() {
        let mut ctx = TransactionContext::new();
        ctx.begin().unwrap();
        
        let original = vec![1, 2, 3];
        ctx.save_original(1, original.clone());
        
        // Second save should not overwrite
        ctx.save_original(1, vec![4, 5, 6]);
        
        assert_eq!(ctx.get_original_data(1), Some(&original));
    }
    
    #[test]
    fn test_rollback_returns_originals() {
        let mut ctx = TransactionContext::new();
        ctx.begin().unwrap();
        
        ctx.save_original(1, vec![1, 2, 3]);
        ctx.save_original(2, vec![4, 5, 6]);
        
        let originals = ctx.rollback();
        
        assert_eq!(originals.len(), 2);
        assert_eq!(originals.get(&1), Some(&vec![1, 2, 3]));
        assert!(!ctx.is_active());
    }
}

