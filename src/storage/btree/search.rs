/// B+Tree search operations
/// 
/// Implements point lookup by key

use crate::error::{Error, Result};
use crate::storage::{Pager, PageId};
use crate::storage::record::Record;
use super::{BTree, node::BTreeNode};

/// Search for a record by exact key match
pub fn search(btree: &BTree, pager: &mut Pager, key: &[u8]) -> Result<Record> {
    let mut page_id = btree.root_page_id();
    
    loop {
        let page = pager.read_page(page_id)?;
        let node = BTreeNode::from_page(page);
        
        if node.is_leaf()? {
            // Search in leaf node
            return search_leaf(&node, key);
        } else {
            // Navigate to child
            page_id = find_child_page(&node, key)?;
        }
    }
}

/// Search for key in a leaf node
fn search_leaf(node: &BTreeNode, key: &[u8]) -> Result<Record> {
    let cell_count = node.cell_count()?;
    
    for i in 0..cell_count {
        let cell = node.get_leaf_cell(i)?;
        if cell.key.as_slice() == key {
            return Ok(cell.record);
        }
        if cell.key.as_slice() > key {
            // Keys are sorted, won't find it
            break;
        }
    }
    
    Err(Error::NotFound)
}

/// Find which child page should contain the key
fn find_child_page(node: &BTreeNode, key: &[u8]) -> Result<PageId> {
    let cell_count = node.cell_count()?;
    
    if cell_count == 0 {
        return node.right_child();
    }
    
    for i in 0..cell_count {
        let cell = node.get_interior_cell(i)?;
        if key < cell.key.as_slice() {
            return Ok(cell.left_child);
        }
    }
    
    // Key is >= all keys, use rightmost child
    node.right_child()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::btree::BTree;
    use crate::storage::record::Value;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_search_single_record() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert a record
        let key = vec![1, 2, 3];
        let record = Record::new(key.clone(), vec![Value::Integer(42)]);
        btree.insert(&mut pager, record).unwrap();
        
        // Search for it
        let found = btree.search(&mut pager, &key).unwrap();
        assert_eq!(found.key, key);
        assert_eq!(found.values[0], Value::Integer(42));
    }
    
    #[test]
    fn test_search_not_found() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let btree = BTree::new(&mut pager).unwrap();
        
        // Search for non-existent key
        let result = btree.search(&mut pager, &[1, 2, 3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::NotFound);
    }
}

