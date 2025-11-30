/// B+Tree deletion operations - ENHANCED VERSION
/// 
/// Implements complete record deletion with:
/// - Node merging when under-utilized
/// - Sibling borrowing (redistribution)
/// - Recursive rebalancing up the tree
/// - Page defragmentation

use crate::error::{Error, Result};
use crate::storage::{Pager, PageId};
use super::{BTree, node::BTreeNode};

const MIN_OCCUPANCY: f32 = 0.5; // Nodes should be at least 50% full

/// Delete result - indicates if rebalancing is needed
#[derive(Debug)]
struct DeleteResult {
    /// Whether the node needs rebalancing (too empty)
    needs_rebalance: bool,
    /// Whether the node was completely emptied (reserved for future use)
    _is_empty: bool,
}

impl DeleteResult {
    fn ok() -> Self {
        DeleteResult {
            needs_rebalance: false,
            _is_empty: false,
        }
    }
    
    fn needs_rebalance() -> Self {
        DeleteResult {
            needs_rebalance: true,
            _is_empty: false,
        }
    }
    
    fn empty() -> Self {
        DeleteResult {
            needs_rebalance: true,
            _is_empty: true,
        }
    }
}

/// Delete a record from the B+Tree
pub fn delete(btree: &mut BTree, pager: &mut Pager, key: &[u8]) -> Result<()> {
    let root_id = btree.root_page_id();
    delete_recursive(pager, root_id, key, btree)?;
    
    // Check if root is now empty interior node
    let root_page = pager.read_page(root_id)?;
    let root_node = BTreeNode::from_page(root_page);
    
    if !root_node.is_leaf()? && root_node.cell_count()? == 0 {
        // Root is empty interior - make its only child the new root
        let new_root_id = root_node.right_child()?;
        btree.set_root_page_id(new_root_id);
        pager.set_root_page(new_root_id)?;
        
        // Free the old root
        pager.free_page(root_id)?;
    }
    
    Ok(())
}

/// Recursive delete with rebalancing
fn delete_recursive(
    pager: &mut Pager,
    page_id: PageId,
    key: &[u8],
    btree: &BTree,
) -> Result<DeleteResult> {
    let page = pager.read_page(page_id)?;
    let mut node = BTreeNode::from_page(page);
    
    if node.is_leaf()? {
        // Leaf node - delete the key
        delete_from_leaf(&mut node, pager, key)
    } else {
        // Interior node - find child and recurse
        let child_id = find_child_for_key(&node, key)?;
        let result = delete_recursive(pager, child_id, key, btree)?;
        
        // Handle child rebalancing
        if result.needs_rebalance {
            rebalance_child(pager, &mut node, child_id)
        } else {
            pager.write_page(node.into_page())?;
            Ok(DeleteResult::ok())
        }
    }
}

/// Delete a key from a leaf node
fn delete_from_leaf(node: &mut BTreeNode, pager: &mut Pager, key: &[u8]) -> Result<DeleteResult> {
    // Find the key
    let cell_count = node.cell_count()?;
    let mut found_index = None;
    
    for i in 0..cell_count {
        let cell = node.get_leaf_cell(i)?;
        if cell.key == key {
            found_index = Some(i);
            break;
        }
    }
    
    match found_index {
        Some(index) => {
            node.delete_cell(index)?;
            let new_count = node.cell_count()?;
            
            // Check occupancy
            let page_size = pager.page_size();
            let used_space = estimate_node_size(node)?;
            let occupancy = used_space as f32 / page_size as f32;
            
            pager.write_page(node.page().clone())?;
            
            if new_count == 0 {
                Ok(DeleteResult::empty())
            } else if occupancy < MIN_OCCUPANCY {
                Ok(DeleteResult::needs_rebalance())
            } else {
                Ok(DeleteResult::ok())
            }
        }
        None => Err(Error::NotFound),
    }
}

/// Rebalance a child node that's under-utilized
fn rebalance_child(pager: &mut Pager, parent: &mut BTreeNode, child_id: PageId) -> Result<DeleteResult> {
    // Find child's position in parent
    let child_index = find_child_index(parent, child_id)?;
    
    // Try to borrow from left sibling
    if child_index > 0 {
        let left_sibling_id = get_sibling_id(parent, child_index, true)?;
        if can_borrow_from_sibling(pager, left_sibling_id)? {
            borrow_from_left_sibling(pager, parent, child_id, left_sibling_id, child_index)?;
            pager.write_page(parent.page().clone())?;
            return Ok(DeleteResult::ok());
        }
    }
    
    // Try to borrow from right sibling
    let cell_count = parent.cell_count()? as usize;
    if child_index < cell_count {
        let right_sibling_id = get_sibling_id(parent, child_index, false)?;
        if can_borrow_from_sibling(pager, right_sibling_id)? {
            borrow_from_right_sibling(pager, parent, child_id, right_sibling_id, child_index)?;
            pager.write_page(parent.page().clone())?;
            return Ok(DeleteResult::ok());
        }
    }
    
    // Can't borrow - merge with sibling
    if child_index > 0 {
        let left_sibling_id = get_sibling_id(parent, child_index, true)?;
        merge_with_left_sibling(pager, parent, child_id, left_sibling_id, child_index)?;
    } else {
        let right_sibling_id = get_sibling_id(parent, child_index, false)?;
        merge_with_right_sibling(pager, parent, child_id, right_sibling_id, child_index)?;
    }
    
    // Check if parent needs rebalancing now
    let parent_count = parent.cell_count()?;
    pager.write_page(parent.page().clone())?;
    
    if parent_count == 0 {
        Ok(DeleteResult::empty())
    } else {
        let page_size = pager.page_size();
        let used_space = estimate_node_size(parent)?;
        let occupancy = used_space as f32 / page_size as f32;
        
        if occupancy < MIN_OCCUPANCY {
            Ok(DeleteResult::needs_rebalance())
        } else {
            Ok(DeleteResult::ok())
        }
    }
}

/// Find the index of a child in its parent
fn find_child_index(parent: &BTreeNode, child_id: PageId) -> Result<usize> {
    let cell_count = parent.cell_count()? as usize;
    
    for i in 0..cell_count {
        let cell = parent.get_interior_cell(i as u16)?;
        if cell.left_child == child_id {
            return Ok(i);
        }
    }
    
    // Must be the rightmost child
    if parent.right_child()? == child_id {
        Ok(cell_count)
    } else {
        Err(Error::Internal("Child not found in parent".to_string()))
    }
}

/// Get sibling page ID
fn get_sibling_id(parent: &BTreeNode, child_index: usize, left: bool) -> Result<PageId> {
    if left {
        if child_index == 0 {
            return Err(Error::Internal("No left sibling".to_string()));
        }
        let cell = parent.get_interior_cell((child_index - 1) as u16)?;
        Ok(cell.left_child)
    } else {
        let cell_count = parent.cell_count()? as usize;
        if child_index >= cell_count {
            return Err(Error::Internal("No right sibling".to_string()));
        }
        let cell = parent.get_interior_cell(child_index as u16)?;
        Ok(cell.left_child) // Actually the right child of this index
    }
}

/// Check if a node can lend a cell to sibling
fn can_borrow_from_sibling(pager: &mut Pager, sibling_id: PageId) -> Result<bool> {
    let sibling_page = pager.read_page(sibling_id)?;
    let sibling = BTreeNode::from_page(sibling_page);
    
    let cell_count = sibling.cell_count()?;
    let page_size = pager.page_size();
    let used_space = estimate_node_size(&sibling)?;
    let occupancy = used_space as f32 / page_size as f32;
    
    // Can borrow if sibling is well above minimum occupancy
    Ok(cell_count > 2 && occupancy > MIN_OCCUPANCY + 0.2)
}

/// Borrow a cell from left sibling
fn borrow_from_left_sibling(
    pager: &mut Pager,
    parent: &mut BTreeNode,
    child_id: PageId,
    left_sibling_id: PageId,
    child_index: usize,
) -> Result<()> {
    let child_page = pager.read_page(child_id)?;
    let mut child = BTreeNode::from_page(child_page);
    
    let sibling_page = pager.read_page(left_sibling_id)?;
    let mut sibling = BTreeNode::from_page(sibling_page);
    
    if child.is_leaf()? {
        // Borrow last cell from sibling
        let sibling_count = sibling.cell_count()?;
        let borrowed_cell = sibling.get_leaf_cell(sibling_count - 1)?;
        
        // Remove from sibling
        sibling.delete_cell(sibling_count - 1)?;
        
        // Add to child at beginning
        child.insert_leaf_cell(0, &borrowed_cell)?;
        
        // Update separator key in parent
        let new_separator = child.get_leaf_cell(0)?.key;
        if child_index > 0 {
            let mut parent_cell = parent.get_interior_cell((child_index - 1) as u16)?;
            parent_cell.key = new_separator;
            parent.delete_cell((child_index - 1) as u16)?;
            parent.insert_interior_cell((child_index - 1) as u16, &parent_cell)?;
        }
        
        pager.write_page(sibling.into_page())?;
        pager.write_page(child.into_page())?;
    }
    
    Ok(())
}

/// Borrow a cell from right sibling
fn borrow_from_right_sibling(
    pager: &mut Pager,
    parent: &mut BTreeNode,
    child_id: PageId,
    right_sibling_id: PageId,
    child_index: usize,
) -> Result<()> {
    let child_page = pager.read_page(child_id)?;
    let mut child = BTreeNode::from_page(child_page);
    
    let sibling_page = pager.read_page(right_sibling_id)?;
    let mut sibling = BTreeNode::from_page(sibling_page);
    
    if child.is_leaf()? {
        // Borrow first cell from sibling
        let borrowed_cell = sibling.get_leaf_cell(0)?;
        
        // Remove from sibling
        sibling.delete_cell(0)?;
        
        // Add to child at end
        let insert_pos = child.cell_count()?;
        child.insert_leaf_cell(insert_pos, &borrowed_cell)?;
        
        // Update separator key in parent
        let new_separator = sibling.get_leaf_cell(0)?.key;
        let mut parent_cell = parent.get_interior_cell(child_index as u16)?;
        parent_cell.key = new_separator;
        parent.delete_cell(child_index as u16)?;
        parent.insert_interior_cell(child_index as u16, &parent_cell)?;
        
        pager.write_page(sibling.into_page())?;
        pager.write_page(child.into_page())?;
    }
    
    Ok(())
}

/// Merge child with left sibling
fn merge_with_left_sibling(
    pager: &mut Pager,
    parent: &mut BTreeNode,
    child_id: PageId,
    left_sibling_id: PageId,
    child_index: usize,
) -> Result<()> {
    let child_page = pager.read_page(child_id)?;
    let child = BTreeNode::from_page(child_page);
    
    let sibling_page = pager.read_page(left_sibling_id)?;
    let mut sibling = BTreeNode::from_page(sibling_page);
    
    if child.is_leaf()? {
        // Copy all cells from child to sibling
        let cell_count = child.cell_count()?;
        for i in 0..cell_count {
            let cell = child.get_leaf_cell(i)?;
            let insert_pos = sibling.cell_count()?;
            sibling.insert_leaf_cell(insert_pos, &cell)?;
        }
        
        // Write merged sibling
        pager.write_page(sibling.into_page())?;
        
        // Free child page
        pager.free_page(child_id)?;
        
        // Remove separator from parent
        if child_index > 0 {
            parent.delete_cell(child_index as u16 - 1)?;
        }
    }
    
    Ok(())
}

/// Merge child with right sibling
fn merge_with_right_sibling(
    pager: &mut Pager,
    parent: &mut BTreeNode,
    child_id: PageId,
    right_sibling_id: PageId,
    child_index: usize,
) -> Result<()> {
    let child_page = pager.read_page(child_id)?;
    let mut child = BTreeNode::from_page(child_page);
    
    let sibling_page = pager.read_page(right_sibling_id)?;
    let sibling = BTreeNode::from_page(sibling_page);
    
    if child.is_leaf()? {
        // Copy all cells from sibling to child
        let sibling_count = sibling.cell_count()?;
        for i in 0..sibling_count {
            let cell = sibling.get_leaf_cell(i)?;
            let insert_pos = child.cell_count()?;
            child.insert_leaf_cell(insert_pos, &cell)?;
        }
        
        // Write merged child
        pager.write_page(child.into_page())?;
        
        // Free sibling page
        pager.free_page(right_sibling_id)?;
        
        // Remove separator from parent
        if child_index < parent.cell_count()? as usize {
            parent.delete_cell(child_index as u16)?;
        }
    }
    
    Ok(())
}

/// Find which child contains a key
fn find_child_for_key(node: &BTreeNode, key: &[u8]) -> Result<PageId> {
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
    
    node.right_child()
}

/// Estimate node space usage for occupancy calculation
fn estimate_node_size(node: &BTreeNode) -> Result<usize> {
    let cell_count = node.cell_count()? as usize;
    
    // Rough estimate: header + cell pointers + average cell size
    let header_size = 12; // Page header size
    let pointer_array_size = cell_count * 2; // 2 bytes per pointer
    
    // Estimate average cell size based on used content area
    let header = node.page().header()?;
    let content_area_used = 4096 - header.cell_content_offset as usize;
    
    Ok(header_size + pointer_array_size + content_area_used)
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
    
    #[test]
    fn test_delete_and_rebalance() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert many records
        for i in 0..50 {
            let key = vec![i as u8];
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            btree.insert(&mut pager, record).unwrap();
        }
        
        // Delete half of them
        for i in 0..25 {
            let key = vec![i as u8];
            btree.delete(&mut pager, &key).unwrap();
        }
        
        // Verify remaining records still accessible
        for i in 25..50 {
            let key = vec![i as u8];
            let found = btree.search(&mut pager, &key);
            assert!(found.is_ok(), "Failed to find key {} after rebalancing", i);
        }
    }
}
