/// B+Tree implementation for table storage
/// 
/// The B+Tree is the core data structure for storing records in DeepSQL.
/// It provides ordered key-value storage with efficient:
/// - Insertion
/// - Deletion
/// - Range scans
/// - Point lookups

pub mod node;
/// Cursor for scanning B+Tree
pub mod cursor;
/// Insert operations
pub mod insert;
/// Delete operations
pub mod delete;
/// Search operations
pub mod search;

use crate::error::{Error, Result};
use crate::storage::{Pager, PageId, PageType};
use crate::storage::record::Record;

pub use cursor::Cursor;
pub use node::BTreeNode;

/// B+Tree structure
pub struct BTree {
    /// Root page ID
    root_page_id: PageId,
}

impl BTree {
    /// Create a new B+Tree
    pub fn new(pager: &mut Pager) -> Result<Self> {
        // Allocate root page as a leaf
        let root_page = pager.allocate_page(PageType::Leaf)?;
        let root_page_id = root_page.id;
        
        // Set as database root
        pager.set_root_page(root_page_id)?;
        
        Ok(BTree { root_page_id })
    }
    
    /// Open an existing B+Tree from root page
    pub fn open(root_page_id: PageId) -> Result<Self> {
        if root_page_id == 0 {
            return Err(Error::BTreeError("Invalid root page ID".to_string()));
        }
        
        Ok(BTree { root_page_id })
    }
    
    /// Get root page ID
    pub fn root_page_id(&self) -> PageId {
        self.root_page_id
    }
    
    /// Set root page ID (used during splits)
    pub fn set_root_page_id(&mut self, page_id: PageId) {
        self.root_page_id = page_id;
    }
    
    /// Insert a record into the B+Tree
    pub fn insert(&mut self, pager: &mut Pager, record: Record) -> Result<()> {
        insert::insert(self, pager, record)
    }
    
    /// Delete a record from the B+Tree
    pub fn delete(&mut self, pager: &mut Pager, key: &[u8]) -> Result<()> {
        delete::delete(self, pager, key)
    }
    
    /// Search for a record by key
    pub fn search(&self, pager: &mut Pager, key: &[u8]) -> Result<Record> {
        search::search(self, pager, key)
    }
    
    /// Create a cursor for scanning the B+Tree
    pub fn cursor(&self, pager: &mut Pager) -> Result<Cursor> {
        Cursor::new(pager, self.root_page_id)
    }
}

