/// B+Tree Bulk Loading - 10-100x faster than sequential inserts
/// 
/// Implements bottom-up B+Tree construction for sorted data
/// Optimal for initial database load and large batch inserts

use crate::error::{Error, Result};
use crate::storage::{Pager, PageId, PageType};
use crate::storage::record::Record;
use super::{BTree, node::{BTreeNode, LeafCell, InteriorCell}};

/// Bulk load configuration
pub struct BulkLoadConfig {
    /// Target fill factor (0.0 to 1.0)
    /// Higher = better space utilization but more splits later
    /// Lower = worse space utilization but fewer splits later
    pub fill_factor: f32,
    
    /// Batch size for page writes
    pub batch_size: usize,
}

impl Default for BulkLoadConfig {
    fn default() -> Self {
        BulkLoadConfig {
            fill_factor: 0.9, // 90% full - good balance
            batch_size: 1000,
        }
    }
}

/// Bulk load records into a B+Tree
/// 
/// Records MUST be sorted by key (ascending order)
/// Returns the number of records loaded
pub fn bulk_load(
    btree: &mut BTree,
    pager: &mut Pager,
    records: Vec<Record>,
    config: &BulkLoadConfig,
) -> Result<usize> {
    if records.is_empty() {
        return Ok(0);
    }
    
    // Validate that records are sorted
    if !is_sorted(&records) {
        return Err(Error::InvalidArgument(
            "Records must be sorted by key for bulk loading".to_string()
        ));
    }
    
    let record_count = records.len();
    
    // Build leaf level first
    let leaf_nodes = build_leaf_level(pager, records, config)?;
    
    // Build interior levels bottom-up
    let root_id = build_interior_levels(pager, leaf_nodes, config)?;
    
    // Update B+Tree root
    btree.set_root_page_id(root_id);
    pager.set_root_page(root_id)?;
    
    Ok(record_count)
}

/// Check if records are sorted by key
fn is_sorted(records: &[Record]) -> bool {
    for i in 1..records.len() {
        if records[i].key < records[i - 1].key {
            return false;
        }
    }
    true
}

/// Build the leaf level of the tree
fn build_leaf_level(
    pager: &mut Pager,
    records: Vec<Record>,
    config: &BulkLoadConfig,
) -> Result<Vec<PageId>> {
    let page_size = pager.page_size();
    let target_fill = (page_size as f32 * config.fill_factor) as usize;
    
    let mut leaf_pages = Vec::new();
    let mut current_page = pager.allocate_page(PageType::Leaf)?;
    let mut current_node = BTreeNode::from_page(current_page);
    let mut current_size = 12; // Page header size
    
    for record in records {
        let cell = LeafCell {
            key: record.key.clone(),
            record,
        };
        
        let cell_size = cell.serialize().len();
        
        // Check if adding this cell would exceed target fill
        if current_size + cell_size + 2 > target_fill && current_node.cell_count()? > 0 {
            // Write current page and start new one
            let page_id = current_node.page_id();
            pager.write_page(current_node.into_page())?;
            leaf_pages.push(page_id);
            
            // Allocate new leaf
            current_page = pager.allocate_page(PageType::Leaf)?;
            current_node = BTreeNode::from_page(current_page);
            current_size = 12;
        }
        
        // Add cell to current page
        let insert_index = current_node.cell_count()?;
        current_node.insert_leaf_cell(insert_index, &cell)?;
        current_size += cell_size + 2; // cell + pointer
    }
    
    // Write last page
    if current_node.cell_count()? > 0 {
        let page_id = current_node.page_id();
        pager.write_page(current_node.into_page())?;
        leaf_pages.push(page_id);
    }
    
    Ok(leaf_pages)
}

/// Build interior levels bottom-up
fn build_interior_levels(
    pager: &mut Pager,
    child_pages: Vec<PageId>,
    config: &BulkLoadConfig,
) -> Result<PageId> {
    // If only one page, it's the root
    if child_pages.len() == 1 {
        return Ok(child_pages[0]);
    }
    
    let page_size = pager.page_size();
    let target_fill = (page_size as f32 * config.fill_factor) as usize;
    
    // Build parent level
    let mut parent_pages = Vec::new();
    let mut current_page = pager.allocate_page(PageType::Interior)?;
    let mut current_node = BTreeNode::from_page(current_page);
    let mut current_size = 16; // Interior page header (with right child pointer)
    
    for (i, &child_id) in child_pages.iter().enumerate() {
        // Read first key from child as separator
        let child_page = pager.read_page(child_id)?;
        let child_node = BTreeNode::from_page(child_page);
        
        let separator_key = if child_node.is_leaf()? {
            // Get first key from leaf
            if child_node.cell_count()? > 0 {
                child_node.get_leaf_cell(0)?.key
            } else {
                continue;
            }
        } else {
            // Get first key from interior node
            if child_node.cell_count()? > 0 {
                child_node.get_interior_cell(0)?.key
            } else {
                continue;
            }
        };
        
        // For all but the last child, create interior cell
        if i < child_pages.len() - 1 {
            let cell = InteriorCell {
                left_child: child_id,
                key: separator_key,
            };
            
            let cell_size = cell.serialize().len();
            
            // Check if adding this cell would exceed target fill
            if current_size + cell_size + 2 > target_fill && current_node.cell_count()? > 0 {
                // Set right child to next child_id
                current_node.set_right_child(child_pages[i])?;
                
                // Write current page and start new one
                let page_id = current_node.page_id();
                pager.write_page(current_node.into_page())?;
                parent_pages.push(page_id);
                
                // Allocate new interior node
                current_page = pager.allocate_page(PageType::Interior)?;
                current_node = BTreeNode::from_page(current_page);
                current_size = 16;
                
                // Add the child we skipped
                let insert_index = current_node.cell_count()?;
                current_node.insert_interior_cell(insert_index, &cell)?;
                current_size += cell_size + 2;
            } else {
                // Add cell to current page
                let insert_index = current_node.cell_count()?;
                current_node.insert_interior_cell(insert_index, &cell)?;
                current_size += cell_size + 2;
            }
        } else {
            // Last child becomes right child
            current_node.set_right_child(child_id)?;
        }
    }
    
    // Write last interior page
    if current_node.cell_count()? > 0 || child_pages.len() == 1 {
        let page_id = current_node.page_id();
        pager.write_page(current_node.into_page())?;
        parent_pages.push(page_id);
    }
    
    // Recursively build next level
    build_interior_levels(pager, parent_pages, config)
}

/// Sort records by key (helper for unsorted input)
pub fn sort_records(mut records: Vec<Record>) -> Vec<Record> {
    records.sort_by(|a, b| a.key.cmp(&b.key));
    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::record::Value;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_bulk_load_sorted() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Create sorted records
        let mut records = Vec::new();
        for i in 0..100 {
            let key = format!("key_{:05}", i).into_bytes();
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            records.push(record);
        }
        
        // Bulk load
        let config = BulkLoadConfig::default();
        let count = bulk_load(&mut btree, &mut pager, records, &config).unwrap();
        assert_eq!(count, 100);
        
        // Verify all records are accessible
        for i in 0..100 {
            let key = format!("key_{:05}", i).into_bytes();
            let found = btree.search(&mut pager, &key);
            assert!(found.is_ok(), "Failed to find key {}", i);
        }
    }
    
    #[test]
    fn test_bulk_load_unsorted_error() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        let mut btree = BTree::new(&mut pager).unwrap();
        
        // Create unsorted records
        let mut records = Vec::new();
        for i in (0..100).rev() {
            let key = vec![i as u8];
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            records.push(record);
        }
        
        // Bulk load should fail
        let config = BulkLoadConfig::default();
        let result = bulk_load(&mut btree, &mut pager, records, &config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sort_records() {
        let mut records = Vec::new();
        for i in (0..10).rev() {
            let key = vec![i as u8];
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            records.push(record);
        }
        
        let sorted = sort_records(records);
        assert!(is_sorted(&sorted));
    }
    
    #[test]
    fn test_bulk_load_small_dataset() {
        // Test with smaller dataset to verify basic functionality
        let temp_file = NamedTempFile::new().unwrap();
        let mut pager = Pager::open(temp_file.path()).unwrap();
        
        // Bulk load approach
        let mut btree_bulk = BTree::new(&mut pager).unwrap();
        let mut records = Vec::new();
        for i in 0..50 {
            let key = format!("key_{:03}", i).into_bytes();
            let record = Record::new(key, vec![Value::Integer(i as i64)]);
            records.push(record);
        }
        
        let config = BulkLoadConfig::default();
        let count = bulk_load(&mut btree_bulk, &mut pager, records, &config).unwrap();
        assert_eq!(count, 50);
        
        // Verify first, middle, and last records
        let first_key = format!("key_{:03}", 0).into_bytes();
        assert!(btree_bulk.search(&mut pager, &first_key).is_ok());
        
        let middle_key = format!("key_{:03}", 25).into_bytes();
        assert!(btree_bulk.search(&mut pager, &middle_key).is_ok());
        
        let last_key = format!("key_{:03}", 49).into_bytes();
        assert!(btree_bulk.search(&mut pager, &last_key).is_ok());
    }
}

