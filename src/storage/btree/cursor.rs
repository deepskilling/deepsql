/// B+Tree cursor for sequential scanning
/// 
/// Provides an iterator-like interface for traversing records in key order

use crate::error::{Error, Result};
use crate::storage::{Pager, PageId};
use crate::storage::record::Record;
use super::node::BTreeNode;

/// Position within a page
#[derive(Debug, Clone)]
struct CursorPosition {
    /// Page ID
    page_id: PageId,
    
    /// Cell index within the page
    cell_index: u16,
}

/// Cursor for scanning a B+Tree
pub struct Cursor {
    /// Current position
    position: Option<CursorPosition>,
    
    /// Root page ID
    root_page_id: PageId,
}

impl Cursor {
    /// Create a new cursor positioned at the start of the tree
    pub fn new(pager: &mut Pager, root_page_id: PageId) -> Result<Self> {
        let mut cursor = Cursor {
            position: None,
            root_page_id,
        };
        
        // Move to first record
        cursor.move_to_first(pager)?;
        
        Ok(cursor)
    }
    
    /// Move cursor to the first record
    pub fn move_to_first(&mut self, pager: &mut Pager) -> Result<()> {
        // Navigate to leftmost leaf
        let mut page_id = self.root_page_id;
        
        loop {
            let page = pager.read_page(page_id)?;
            let node = BTreeNode::from_page(page);
            
            if node.is_leaf()? {
                // Found leftmost leaf
                self.position = Some(CursorPosition {
                    page_id,
                    cell_index: 0,
                });
                break;
            } else {
                // Navigate to leftmost child
                let cell_count = node.cell_count()?;
                if cell_count == 0 {
                    // Empty interior node, use right child
                    page_id = node.right_child()?;
                } else {
                    // Use first cell's left child
                    let cell = node.get_interior_cell(0)?;
                    page_id = cell.left_child;
                }
            }
        }
        
        Ok(())
    }
    
    /// Seek to a specific key (or next key if not found)
    pub fn seek(&mut self, pager: &mut Pager, key: &[u8]) -> Result<()> {
        let mut page_id = self.root_page_id;
        
        loop {
            let page = pager.read_page(page_id)?;
            let node = BTreeNode::from_page(page);
            
            if node.is_leaf()? {
                // Found leaf, find position within it
                let cell_index = node.find_cell_index(key)?;
                self.position = Some(CursorPosition {
                    page_id,
                    cell_index,
                });
                break;
            } else {
                // Navigate to appropriate child
                page_id = self.find_child_page(&node, key)?;
            }
        }
        
        Ok(())
    }
    
    /// Find which child page contains the key
    fn find_child_page(&self, node: &BTreeNode, key: &[u8]) -> Result<PageId> {
        let cell_count = node.cell_count()?;
        
        if cell_count == 0 {
            return node.right_child();
        }
        
        // Binary search for the right child
        for i in 0..cell_count {
            let cell = node.get_interior_cell(i)?;
            if key < &cell.key {
                return Ok(cell.left_child);
            }
        }
        
        // Key is >= all keys, use rightmost child
        node.right_child()
    }
    
    /// Get current record
    pub fn current(&self, pager: &mut Pager) -> Result<Record> {
        let pos = self.position.as_ref()
            .ok_or(Error::NotFound)?;
        
        let page = pager.read_page(pos.page_id)?;
        let node = BTreeNode::from_page(page);
        
        if !node.is_leaf()? {
            return Err(Error::Internal("Cursor not on leaf page".to_string()));
        }
        
        if pos.cell_index >= node.cell_count()? {
            return Err(Error::NotFound);
        }
        
        let cell = node.get_leaf_cell(pos.cell_index)?;
        Ok(cell.record)
    }
    
    /// Move to next record
    pub fn next(&mut self, pager: &mut Pager) -> Result<bool> {
        let pos = self.position.as_mut()
            .ok_or(Error::NotFound)?;
        
        let page = pager.read_page(pos.page_id)?;
        let node = BTreeNode::from_page(page);
        let cell_count = node.cell_count()?;
        
        // Try to move to next cell in current page
        if pos.cell_index + 1 < cell_count {
            pos.cell_index += 1;
            return Ok(true);
        }
        
        // Need to move to next leaf page
        // For simplicity, we'll just mark as end of cursor
        // A full implementation would maintain a stack or sibling pointers
        self.position = None;
        Ok(false)
    }
    
    /// Check if cursor is valid
    pub fn is_valid(&self) -> bool {
        self.position.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::btree::BTree;
    use crate::storage::record::Value;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_cursor_scan() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert some records
        for i in 0..5 {
            let key = vec![i as u8];
            let record = Record::new(key.clone(), vec![Value::Integer(i as i64)]);
            btree.insert(&mut pager, record).unwrap();
        }
        
        // Scan with cursor
        let mut cursor = btree.cursor(&mut pager).unwrap();
        let mut count = 0;
        
        while cursor.is_valid() {
            let _record = cursor.current(&mut pager).unwrap();
            count += 1;
            
            if !cursor.next(&mut pager).unwrap() {
                break;
            }
        }
        
        assert_eq!(count, 5);
    }
}

