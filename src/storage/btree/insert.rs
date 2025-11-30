/// B+Tree insertion operations
/// 
/// Implements record insertion with node splits when necessary

use crate::error::Result;
use crate::storage::{Pager, PageId, PageType};
use crate::storage::record::Record;
use super::{BTree, node::{BTreeNode, LeafCell}};

/// Insert a record into the B+Tree
pub fn insert(btree: &mut BTree, pager: &mut Pager, record: Record) -> Result<()> {
    let root_page_id = btree.root_page_id();
    let root_page = pager.read_page(root_page_id)?;
    let mut root_node = BTreeNode::from_page(root_page);
    
    if root_node.is_leaf()? {
        // Root is a leaf
        insert_into_leaf(pager, &mut root_node, record)?;
        pager.write_page(root_node.into_page())?;
    } else {
        // Root is interior, navigate to leaf
        let leaf_id = find_leaf_for_insert(&mut root_node, pager, &record.key)?;
        let leaf_page = pager.read_page(leaf_id)?;
        let mut leaf_node = BTreeNode::from_page(leaf_page);
        
        insert_into_leaf(pager, &mut leaf_node, record)?;
        pager.write_page(leaf_node.into_page())?;
    }
    
    Ok(())
}

/// Find the leaf node where a record should be inserted
fn find_leaf_for_insert(node: &BTreeNode, pager: &mut Pager, key: &[u8]) -> Result<PageId> {
    let mut page_id = node.page_id();
    
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

/// Insert a record into a leaf node
fn insert_into_leaf(pager: &mut Pager, node: &mut BTreeNode, record: Record) -> Result<()> {
    let cell = LeafCell {
        key: record.key.clone(),
        record,
    };
    
    let cell_size = cell.serialize().len();
    
    // Check if we need to split
    if !node.has_space_for_cell(cell_size)? {
        return split_leaf(pager, node, cell);
    }
    
    // Find insertion position
    let insert_index = node.find_cell_index(&cell.key)?;
    
    // Check if key already exists (update case)
    let cell_count = node.cell_count()?;
    if insert_index < cell_count {
        let existing_cell = node.get_leaf_cell(insert_index)?;
        if existing_cell.key == cell.key {
            // Key exists, delete old and insert new
            node.delete_cell(insert_index)?;
        }
    }
    
    // Insert the cell
    node.insert_leaf_cell(insert_index, &cell)?;
    
    Ok(())
}

/// Split a leaf node when it's full
fn split_leaf(pager: &mut Pager, node: &mut BTreeNode, new_cell: LeafCell) -> Result<()> {
    // Create a new leaf page
    let new_page = pager.allocate_page(PageType::Leaf)?;
    let mut new_node = BTreeNode::from_page(new_page);
    
    // Collect all cells (including the new one)
    let cell_count = node.cell_count()?;
    let mut all_cells = Vec::new();
    
    for i in 0..cell_count {
        all_cells.push(node.get_leaf_cell(i)?);
    }
    
    all_cells.push(new_cell);
    
    // Sort by key
    all_cells.sort_by(|a, b| a.key.cmp(&b.key));
    
    // Split point (middle)
    let split_point = all_cells.len() / 2;
    
    // Clear original node and add first half
    let page_size = pager.page_size();
    node.page_mut().initialize(PageType::Leaf, page_size)?;
    
    for cell in &all_cells[..split_point] {
        let insert_index = node.find_cell_index(&cell.key)?;
        node.insert_leaf_cell(insert_index, cell)?;
    }
    
    // Add second half to new node
    for cell in &all_cells[split_point..] {
        let insert_index = new_node.find_cell_index(&cell.key)?;
        new_node.insert_leaf_cell(insert_index, cell)?;
    }
    
    // Write both nodes
    pager.write_page(node.page().clone())?;
    pager.write_page(new_node.into_page())?;
    
    // TODO: Handle updating parent (or creating new root if needed)
    // For now, this is a simplified implementation
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::record::Value;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_insert_single_record() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        let key = vec![1, 2, 3];
        let record = Record::new(key.clone(), vec![Value::Integer(42)]);
        
        btree.insert(&mut pager, record).unwrap();
        
        // Verify insertion
        let found = btree.search(&mut pager, &key).unwrap();
        assert_eq!(found.key, key);
    }
    
    #[test]
    fn test_insert_multiple_records() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert multiple records
        for i in 0..10 {
            let key = vec![i as u8];
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            btree.insert(&mut pager, record).unwrap();
        }
        
        // Verify all records
        for i in 0..10 {
            let key = vec![i as u8];
            let found = btree.search(&mut pager, &key).unwrap();
            assert_eq!(found.values[0], Value::Integer(i as i64));
        }
    }
    
    #[test]
    fn test_insert_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        let key = vec![1];
        
        // Insert initial record
        let record1 = Record::new(key.clone(), vec![Value::Integer(42)]);
        btree.insert(&mut pager, record1).unwrap();
        
        // Update with new value
        let record2 = Record::new(key.clone(), vec![Value::Integer(100)]);
        btree.insert(&mut pager, record2).unwrap();
        
        // Verify updated value
        let found = btree.search(&mut pager, &key).unwrap();
        assert_eq!(found.values[0], Value::Integer(100));
    }
}

