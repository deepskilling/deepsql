/// B+Tree deletion operations
/// 
/// Implements record deletion with node merging when necessary

use crate::error::{Error, Result};
use crate::storage::Pager;
use super::{BTree, node::BTreeNode};

/// Delete a record from the B+Tree
pub fn delete(btree: &mut BTree, pager: &mut Pager, key: &[u8]) -> Result<()> {
    let leaf_id = find_leaf_for_delete(btree, pager, key)?;
    let leaf_page = pager.read_page(leaf_id)?;
    let mut leaf_node = BTreeNode::from_page(leaf_page);
    
    // Find the key in the leaf
    let cell_count = leaf_node.cell_count()?;
    let mut found_index = None;
    
    for i in 0..cell_count {
        let cell = leaf_node.get_leaf_cell(i)?;
        if cell.key == key {
            found_index = Some(i);
            break;
        }
    }
    
    match found_index {
        Some(index) => {
            leaf_node.delete_cell(index)?;
            pager.write_page(leaf_node.into_page())?;
            Ok(())
        }
        None => Err(Error::NotFound),
    }
}

/// Find the leaf node containing a key
fn find_leaf_for_delete(btree: &BTree, pager: &mut Pager, key: &[u8]) -> Result<u32> {
    let mut page_id = btree.root_page_id();
    
    loop {
        let page = pager.read_page(page_id)?;
        let node = BTreeNode::from_page(page);
        
        if node.is_leaf()? {
            return Ok(page_id);
        }
        
        // Find child
        let cell_count = node.cell_count()?;
        
        if cell_count == 0 {
            page_id = node.right_child()?;
            continue;
        }
        
        let mut found = false;
        for i in 0..cell_count {
            let cell = node.get_interior_cell(i)?;
            if key < &cell.key {
                page_id = cell.left_child;
                found = true;
                break;
            }
        }
        
        if !found {
            page_id = node.right_child()?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::btree::BTree;
    use crate::storage::record::{Record, Value};
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_delete_record() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert a record
        let key = vec![1, 2, 3];
        let record = Record::new(key.clone(), vec![Value::Integer(42)]);
        btree.insert(&mut pager, record).unwrap();
        
        // Verify it exists
        assert!(btree.search(&mut pager, &key).is_ok());
        
        // Delete it
        btree.delete(&mut pager, &key).unwrap();
        
        // Verify it's gone
        assert!(btree.search(&mut pager, &key).is_err());
    }
    
    #[test]
    fn test_delete_nonexistent() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Try to delete non-existent key
        let result = btree.delete(&mut pager, &[1, 2, 3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::NotFound);
    }
    
    #[test]
    fn test_delete_multiple() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert multiple records
        for i in 0..5 {
            let key = vec![i as u8];
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            btree.insert(&mut pager, record).unwrap();
        }
        
        // Delete some of them
        btree.delete(&mut pager, &[1]).unwrap();
        btree.delete(&mut pager, &[3]).unwrap();
        
        // Verify remaining records
        assert!(btree.search(&mut pager, &[0]).is_ok());
        assert!(btree.search(&mut pager, &[1]).is_err());
        assert!(btree.search(&mut pager, &[2]).is_ok());
        assert!(btree.search(&mut pager, &[3]).is_err());
        assert!(btree.search(&mut pager, &[4]).is_ok());
    }
}

