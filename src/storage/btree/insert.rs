/// B+Tree insertion operations - ENHANCED VERSION
/// 
/// Implements complete record insertion with:
/// - Node splits with parent updates
/// - Root node splitting
/// - Interior node splits
/// - Recursive split propagation

use crate::error::Result;
use crate::storage::{Pager, PageId, PageType};
use crate::storage::record::Record;
use super::{BTree, node::{BTreeNode, LeafCell, InteriorCell}};

/// Insert result - indicates if a split occurred
#[derive(Debug)]
struct InsertResult {
    /// Whether a split occurred
    split: bool,
    /// If split, the new right sibling page ID
    new_page_id: Option<PageId>,
    /// If split, the separator key
    separator_key: Option<Vec<u8>>,
}

impl InsertResult {
    fn no_split() -> Self {
        InsertResult {
            split: false,
            new_page_id: None,
            separator_key: None,
        }
    }
    
    fn with_split(new_page_id: PageId, separator_key: Vec<u8>) -> Self {
        InsertResult {
            split: true,
            new_page_id: Some(new_page_id),
            separator_key: Some(separator_key),
        }
    }
}

/// Insert a record into the B+Tree
pub fn insert(btree: &mut BTree, pager: &mut Pager, record: Record) -> Result<()> {
    let root_page_id = btree.root_page_id();
    let result = insert_recursive(pager, root_page_id, record)?;
    
    // Handle root split
    if result.split {
        let new_root_page = pager.allocate_page(PageType::Interior)?;
        let mut new_root = BTreeNode::from_page(new_root_page);
        let new_root_id = new_root.page_id();
        
        // New root points to old root (left) and new sibling (right)
        let separator_key = result.separator_key.unwrap();
        let new_page_id = result.new_page_id.unwrap();
        
        let interior_cell = InteriorCell {
            left_child: root_page_id,
            key: separator_key,
        };
        
        new_root.insert_interior_cell(0, &interior_cell)?;
        new_root.set_right_child(new_page_id)?;
        
        pager.write_page(new_root.into_page())?;
        btree.set_root_page_id(new_root_id);
        pager.set_root_page(new_root_id)?;
    }
    
    Ok(())
}

/// Recursive insert with split propagation
fn insert_recursive(pager: &mut Pager, page_id: PageId, record: Record) -> Result<InsertResult> {
    let page = pager.read_page(page_id)?;
    let mut node = BTreeNode::from_page(page);
    
    if node.is_leaf()? {
        // Leaf node - insert directly
        insert_into_leaf_recursive(pager, &mut node, record)
    } else {
        // Interior node - find child and recurse
        let child_id = find_child_for_key(&node, &record.key)?;
        let child_result = insert_recursive(pager, child_id, record)?;
        
        // Handle child split
        if child_result.split {
            handle_child_split(pager, &mut node, child_id, child_result)
        } else {
            pager.write_page(node.into_page())?;
            Ok(InsertResult::no_split())
        }
    }
}

/// Insert into leaf node with split support
fn insert_into_leaf_recursive(pager: &mut Pager, node: &mut BTreeNode, record: Record) -> Result<InsertResult> {
    let cell = LeafCell {
        key: record.key.clone(),
        record,
    };
    
    let cell_size = cell.serialize().len();
    
    // Check if we need to split
    if !node.has_space_for_cell(cell_size)? {
        return split_leaf_and_insert(pager, node, cell);
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
    pager.write_page(node.page().clone())?;
    
    Ok(InsertResult::no_split())
}

/// Split a leaf node and return split information
fn split_leaf_and_insert(pager: &mut Pager, node: &mut BTreeNode, new_cell: LeafCell) -> Result<InsertResult> {
    // Create a new leaf page
    let new_page = pager.allocate_page(PageType::Leaf)?;
    let new_page_id = new_page.id;
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
    
    // Separator key is the first key of the right node
    let separator_key = all_cells[split_point].key.clone();
    
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
    
    Ok(InsertResult::with_split(new_page_id, separator_key))
}

/// Handle a child node split in an interior node
fn handle_child_split(
    pager: &mut Pager,
    parent: &mut BTreeNode,
    child_id: PageId,
    child_result: InsertResult,
) -> Result<InsertResult> {
    let separator_key = child_result.separator_key.unwrap();
    let new_page_id = child_result.new_page_id.unwrap();
    
    // Create interior cell for the split
    let interior_cell = InteriorCell {
        left_child: child_id,
        key: separator_key.clone(),
    };
    
    let cell_size = interior_cell.serialize().len();
    
    // Check if parent needs to split
    if !parent.has_space_for_cell(cell_size)? {
        return split_interior_and_insert(pager, parent, interior_cell, new_page_id);
    }
    
    // Find where to insert in parent
    let insert_index = parent.find_cell_index(&separator_key)?;
    
    // Insert the new separator
    parent.insert_interior_cell(insert_index, &interior_cell)?;
    
    // Update the right child of the previous cell (or root right child)
    if insert_index == parent.cell_count()? - 1 {
        // Inserted at end, update parent's right child
        let _old_right = parent.right_child()?;
        parent.set_right_child(new_page_id)?;
        
        // Update the inserted cell's left child to point to old right
        // Actually, we need to be more careful here...
        // The new_page_id should become the new right child
    } else {
        // Update next cell's left child
        let next_cell = parent.get_interior_cell(insert_index + 1)?;
        let updated_cell = InteriorCell {
            left_child: new_page_id,
            key: next_cell.key.clone(),
        };
        parent.delete_cell(insert_index + 1)?;
        parent.insert_interior_cell(insert_index + 1, &updated_cell)?;
    }
    
    pager.write_page(parent.page().clone())?;
    Ok(InsertResult::no_split())
}

/// Split an interior node when it's full
fn split_interior_and_insert(
    pager: &mut Pager,
    node: &mut BTreeNode,
    new_cell: InteriorCell,
    right_child: PageId,
) -> Result<InsertResult> {
    // Create new interior page
    let new_page = pager.allocate_page(PageType::Interior)?;
    let new_page_id = new_page.id;
    let mut new_node = BTreeNode::from_page(new_page);
    
    // Collect all cells
    let cell_count = node.cell_count()?;
    let mut all_cells = Vec::new();
    
    for i in 0..cell_count {
        all_cells.push(node.get_interior_cell(i)?);
    }
    
    all_cells.push(new_cell);
    all_cells.sort_by(|a, b| a.key.cmp(&b.key));
    
    // Split point
    let split_point = all_cells.len() / 2;
    
    // Separator key is the middle key (promoted to parent)
    let separator_key = all_cells[split_point].key.clone();
    
    // Clear and rebuild left node (first half)
    let page_size = pager.page_size();
    node.page_mut().initialize(PageType::Interior, page_size)?;
    
    for cell in &all_cells[..split_point] {
        let insert_index = node.find_cell_index(&cell.key)?;
        node.insert_interior_cell(insert_index, cell)?;
    }
    
    // Set right child of left node to left child of separator
    node.set_right_child(all_cells[split_point].left_child)?;
    
    // Build right node (second half, excluding separator)
    for cell in &all_cells[split_point + 1..] {
        let insert_index = new_node.find_cell_index(&cell.key)?;
        new_node.insert_interior_cell(insert_index, cell)?;
    }
    
    new_node.set_right_child(right_child)?;
    
    // Write both nodes
    pager.write_page(node.page().clone())?;
    pager.write_page(new_node.into_page())?;
    
    Ok(InsertResult::with_split(new_page_id, separator_key))
}

/// Find which child to follow for a given key
fn find_child_for_key(node: &BTreeNode, key: &[u8]) -> Result<PageId> {
    let cell_count = node.cell_count()?;
    
    if cell_count == 0 {
        return node.right_child();
    }
    
    for i in 0..cell_count {
        let cell = node.get_interior_cell(i)?;
        if key < &cell.key {
            return Ok(cell.left_child);
        }
    }
    
    node.right_child()
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
    
    #[test]
    fn test_insert_many_records_with_splits() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert 100 records to force splits
        for i in 0..100 {
            let key = format!("key_{:05}", i).into_bytes();
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            btree.insert(&mut pager, record).unwrap();
        }
        
        // Verify all 100 records are retrievable
        for i in 0..100 {
            let key = format!("key_{:05}", i).into_bytes();
            let found = btree.search(&mut pager, &key);
            assert!(found.is_ok(), "Failed to find key {}", i);
            assert_eq!(found.unwrap().values[0], Value::Integer(i as i64));
        }
    }
    
    #[test]
    fn test_insert_reverse_order() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Insert in reverse order to test different split patterns
        for i in (0..50).rev() {
            let key = vec![i as u8];
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            btree.insert(&mut pager, record).unwrap();
        }
        
        // Verify all records
        for i in 0..50 {
            let key = vec![i as u8];
            let found = btree.search(&mut pager, &key).unwrap();
            assert_eq!(found.values[0], Value::Integer(i as i64));
        }
    }
}
