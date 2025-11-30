/// B+Tree node operations
/// 
/// Handles reading and writing B+Tree nodes (leaf and interior pages)

use crate::error::{Error, Result};
use crate::storage::{Page, PageType, PageId};
use crate::storage::page::PageHeader;
use crate::storage::record::{Record, Varint};

/// Cell in a leaf page (contains full record)
#[derive(Debug, Clone)]
pub struct LeafCell {
    /// Record key
    pub key: Vec<u8>,
    
    /// Full record
    pub record: Record,
}

impl LeafCell {
    /// Serialize leaf cell to bytes
    pub fn serialize(&self) -> Vec<u8> {
        self.record.serialize()
    }
    
    /// Deserialize leaf cell from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let record = Record::deserialize(bytes)?;
        let key = record.key.clone();
        Ok(LeafCell { key, record })
    }
}

/// Cell in an interior page (contains key and child pointer)
#[derive(Debug, Clone)]
pub struct InteriorCell {
    /// Key (smallest key in right child subtree)
    pub key: Vec<u8>,
    
    /// Left child page ID
    pub left_child: PageId,
}

impl InteriorCell {
    /// Serialize interior cell to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize left child
        bytes.extend_from_slice(&self.left_child.to_be_bytes());
        
        // Serialize key
        bytes.extend_from_slice(&Varint::encode(self.key.len() as u64));
        bytes.extend_from_slice(&self.key);
        
        bytes
    }
    
    /// Deserialize interior cell from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(Error::BTreeError("Interior cell too short".to_string()));
        }
        
        // Deserialize left child
        let left_child = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        
        // Deserialize key
        let (key_len, consumed) = Varint::decode(&bytes[4..])?;
        let key_offset = 4 + consumed;
        
        if bytes.len() < key_offset + key_len as usize {
            return Err(Error::BTreeError("Interior cell key truncated".to_string()));
        }
        
        let key = bytes[key_offset..key_offset + key_len as usize].to_vec();
        
        Ok(InteriorCell { key, left_child })
    }
}

/// B+Tree node wrapper for page operations
pub struct BTreeNode {
    /// Underlying page
    page: Page,
}

impl BTreeNode {
    /// Create node from page
    pub fn from_page(page: Page) -> Self {
        BTreeNode { page }
    }
    
    /// Get underlying page
    pub fn page(&self) -> &Page {
        &self.page
    }
    
    /// Get mutable page
    pub fn page_mut(&mut self) -> &mut Page {
        &mut self.page
    }
    
    /// Take ownership of the page
    pub fn into_page(self) -> Page {
        self.page
    }
    
    /// Get page ID
    pub fn page_id(&self) -> PageId {
        self.page.id
    }
    
    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> Result<bool> {
        let header = self.page.header()?;
        Ok(header.page_type == PageType::Leaf)
    }
    
    /// Get number of cells
    pub fn cell_count(&self) -> Result<u16> {
        let header = self.page.header()?;
        Ok(header.cell_count)
    }
    
    /// Get leaf cell at index
    pub fn get_leaf_cell(&self, index: u16) -> Result<LeafCell> {
        let pointer = self.page.get_cell_pointer(index)?;
        let cell_data = self.page.get_cell_data(pointer)?;
        LeafCell::deserialize(cell_data)
    }
    
    /// Get interior cell at index
    pub fn get_interior_cell(&self, index: u16) -> Result<InteriorCell> {
        let pointer = self.page.get_cell_pointer(index)?;
        let cell_data = self.page.get_cell_data(pointer)?;
        InteriorCell::deserialize(cell_data)
    }
    
    /// Insert a leaf cell
    pub fn insert_leaf_cell(&mut self, index: u16, cell: &LeafCell) -> Result<()> {
        let cell_data = cell.serialize();
        self.insert_cell(index, &cell_data)
    }
    
    /// Insert an interior cell
    pub fn insert_interior_cell(&mut self, index: u16, cell: &InteriorCell) -> Result<()> {
        let cell_data = cell.serialize();
        self.insert_cell(index, &cell_data)
    }
    
    /// Insert cell data at index
    fn insert_cell(&mut self, index: u16, cell_data: &[u8]) -> Result<()> {
        let mut header = self.page.header()?;
        
        // Calculate where to place the cell content
        let new_content_offset = header.cell_content_offset - cell_data.len() as u16;
        
        // Write cell content
        let content_start = new_content_offset as usize;
        self.page.data[content_start..content_start + cell_data.len()]
            .copy_from_slice(cell_data);
        
        // Shift cell pointers to make room
        for i in (index..header.cell_count).rev() {
            let old_pointer = self.page.get_cell_pointer(i)?;
            self.page.set_cell_pointer(i + 1, old_pointer)?;
        }
        
        // Insert new cell pointer
        self.page.set_cell_pointer(index, new_content_offset)?;
        
        // Update header
        header.cell_count += 1;
        header.cell_content_offset = new_content_offset;
        self.page.set_header(&header)?;
        
        Ok(())
    }
    
    /// Delete cell at index
    pub fn delete_cell(&mut self, index: u16) -> Result<()> {
        let mut header = self.page.header()?;
        
        if index >= header.cell_count {
            return Err(Error::BTreeError("Cell index out of bounds".to_string()));
        }
        
        // Shift cell pointers
        for i in index..header.cell_count - 1 {
            let pointer = self.page.get_cell_pointer(i + 1)?;
            self.page.set_cell_pointer(i, pointer)?;
        }
        
        // Update header
        header.cell_count -= 1;
        self.page.set_header(&header)?;
        
        // Note: We're not reclaiming the space from deleted cells
        // This would require defragmentation
        
        Ok(())
    }
    
    /// Find the index where a key should be inserted (binary search)
    pub fn find_cell_index(&self, key: &[u8]) -> Result<u16> {
        let cell_count = self.cell_count()?;
        
        if cell_count == 0 {
            return Ok(0);
        }
        
        let is_leaf = self.is_leaf()?;
        
        let mut left = 0;
        let mut right = cell_count;
        
        while left < right {
            let mid = left + (right - left) / 2;
            
            let cell_key = if is_leaf {
                self.get_leaf_cell(mid)?.key
            } else {
                self.get_interior_cell(mid)?.key
            };
            
            if cell_key.as_slice() < key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        
        Ok(left)
    }
    
    /// Get right-most child pointer (for interior nodes)
    pub fn right_child(&self) -> Result<PageId> {
        let header = self.page.header()?;
        Ok(header.right_child)
    }
    
    /// Set right-most child pointer (for interior nodes)
    pub fn set_right_child(&mut self, page_id: PageId) -> Result<()> {
        let mut header = self.page.header()?;
        header.right_child = page_id;
        self.page.set_header(&header)?;
        Ok(())
    }
    
    /// Check if node has space for a new cell
    pub fn has_space_for_cell(&self, cell_size: usize) -> Result<bool> {
        let header = self.page.header()?;
        
        // Calculate free space
        let pointer_array_end = PageHeader::SIZE + (header.cell_count as usize + 1) * 2;
        let content_start = header.cell_content_offset as usize;
        
        let free_space = if content_start > pointer_array_end {
            content_start - pointer_array_end
        } else {
            0
        };
        
        // Need space for cell pointer (2 bytes) + cell content
        Ok(free_space >= 2 + cell_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::record::Value;
    
    #[test]
    fn test_leaf_cell_serialization() {
        let record = Record::new(
            vec![1, 2, 3],
            vec![Value::Integer(42)],
        );
        
        let cell = LeafCell {
            key: vec![1, 2, 3],
            record,
        };
        
        let serialized = cell.serialize();
        let deserialized = LeafCell::deserialize(&serialized).unwrap();
        
        assert_eq!(cell.key, deserialized.key);
        assert_eq!(cell.record.key, deserialized.record.key);
    }
    
    #[test]
    fn test_interior_cell_serialization() {
        let cell = InteriorCell {
            key: vec![1, 2, 3],
            left_child: 42,
        };
        
        let serialized = cell.serialize();
        let deserialized = InteriorCell::deserialize(&serialized).unwrap();
        
        assert_eq!(cell.key, deserialized.key);
        assert_eq!(cell.left_child, deserialized.left_child);
    }
}

