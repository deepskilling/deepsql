/// Page types and structures
/// 
/// DeepSQL uses fixed-size pages as the basic unit of storage.
/// There are several types of pages for different purposes.

use crate::error::{Error, Result};

/// Page ID type (1-indexed, 0 means NULL)
pub type PageId = u32;

/// Page types in DeepSQL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    /// Header page (always page 1)
    Header = 0,
    
    /// Leaf page in B+Tree (contains actual data records)
    Leaf = 1,
    
    /// Interior page in B+Tree (contains keys and pointers)
    Interior = 2,
    
    /// Overflow page (for large records)
    Overflow = 3,
    
    /// Free page (on freelist)
    Free = 4,
}

impl PageType {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(PageType::Header),
            1 => Ok(PageType::Leaf),
            2 => Ok(PageType::Interior),
            3 => Ok(PageType::Overflow),
            4 => Ok(PageType::Free),
            _ => Err(Error::InvalidPage(format!("Invalid page type: {}", value))),
        }
    }
}

/// Page header structure (common to all page types except Header)
#[derive(Debug, Clone)]
pub struct PageHeader {
    /// Type of this page
    pub page_type: PageType,
    
    /// Number of cells (records/keys) in this page
    pub cell_count: u16,
    
    /// Offset to start of cell content area
    pub cell_content_offset: u16,
    
    /// Number of fragmented free bytes
    pub fragmented_free_bytes: u16,
    
    /// Right-most child pointer (for interior pages only)
    pub right_child: PageId,
}

impl PageHeader {
    /// Size of page header in bytes
    pub const SIZE: usize = 12;
    
    /// Create a new page header
    pub fn new(page_type: PageType) -> Self {
        PageHeader {
            page_type,
            cell_count: 0,
            cell_content_offset: 0,
            fragmented_free_bytes: 0,
            right_child: 0,
        }
    }
    
    /// Serialize page header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        
        bytes.push(self.page_type as u8);
        bytes.extend_from_slice(&self.cell_count.to_be_bytes());
        bytes.extend_from_slice(&self.cell_content_offset.to_be_bytes());
        bytes.extend_from_slice(&self.fragmented_free_bytes.to_be_bytes());
        bytes.extend_from_slice(&self.right_child.to_be_bytes());
        
        bytes
    }
    
    /// Deserialize page header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(Error::InvalidPage("Page header too short".to_string()));
        }
        
        let page_type = PageType::from_u8(bytes[0])?;
        let cell_count = u16::from_be_bytes([bytes[1], bytes[2]]);
        let cell_content_offset = u16::from_be_bytes([bytes[3], bytes[4]]);
        let fragmented_free_bytes = u16::from_be_bytes([bytes[5], bytes[6]]);
        let right_child = u32::from_be_bytes([
            bytes[7], bytes[8], bytes[9], bytes[10]
        ]);
        
        Ok(PageHeader {
            page_type,
            cell_count,
            cell_content_offset,
            fragmented_free_bytes,
            right_child,
        })
    }
}

/// Cell pointer (offset to cell content)
pub type CellPointer = u16;

/// In-memory representation of a page
#[derive(Debug, Clone)]
pub struct Page {
    /// Page ID
    pub id: PageId,
    
    /// Raw page data
    pub data: Vec<u8>,
    
    /// Whether this page has been modified
    pub dirty: bool,
}

impl Page {
    /// Create a new empty page
    pub fn new(id: PageId, size: usize) -> Self {
        Page {
            id,
            data: vec![0; size],
            dirty: false,
        }
    }
    
    /// Create a page from raw data
    pub fn from_data(id: PageId, data: Vec<u8>) -> Self {
        Page {
            id,
            data,
            dirty: false,
        }
    }
    
    /// Get page header
    pub fn header(&self) -> Result<PageHeader> {
        PageHeader::from_bytes(&self.data)
    }
    
    /// Set page header
    pub fn set_header(&mut self, header: &PageHeader) -> Result<()> {
        let bytes = header.to_bytes();
        if bytes.len() > self.data.len() {
            return Err(Error::InvalidPage("Page too small for header".to_string()));
        }
        self.data[..bytes.len()].copy_from_slice(&bytes);
        self.dirty = true;
        Ok(())
    }
    
    /// Get cell pointer at index
    pub fn get_cell_pointer(&self, index: u16) -> Result<CellPointer> {
        let offset = PageHeader::SIZE + (index as usize) * 2;
        if offset + 2 > self.data.len() {
            return Err(Error::InvalidPage("Cell pointer out of bounds".to_string()));
        }
        
        Ok(u16::from_be_bytes([
            self.data[offset],
            self.data[offset + 1],
        ]))
    }
    
    /// Set cell pointer at index
    pub fn set_cell_pointer(&mut self, index: u16, pointer: CellPointer) -> Result<()> {
        let offset = PageHeader::SIZE + (index as usize) * 2;
        if offset + 2 > self.data.len() {
            return Err(Error::InvalidPage("Cell pointer out of bounds".to_string()));
        }
        
        let bytes = pointer.to_be_bytes();
        self.data[offset] = bytes[0];
        self.data[offset + 1] = bytes[1];
        self.dirty = true;
        Ok(())
    }
    
    /// Get cell data at pointer offset
    pub fn get_cell_data(&self, pointer: CellPointer) -> Result<&[u8]> {
        let offset = pointer as usize;
        if offset >= self.data.len() {
            return Err(Error::InvalidPage("Cell data out of bounds".to_string()));
        }
        Ok(&self.data[offset..])
    }
    
    /// Get mutable cell data at pointer offset
    pub fn get_cell_data_mut(&mut self, pointer: CellPointer) -> Result<&mut [u8]> {
        let offset = pointer as usize;
        if offset >= self.data.len() {
            return Err(Error::InvalidPage("Cell data out of bounds".to_string()));
        }
        self.dirty = true;
        Ok(&mut self.data[offset..])
    }
    
    /// Initialize page as a specific type
    pub fn initialize(&mut self, page_type: PageType, page_size: usize) -> Result<()> {
        let mut header = PageHeader::new(page_type);
        header.cell_content_offset = page_size as u16;
        self.set_header(&header)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_page_type_conversion() {
        assert_eq!(PageType::from_u8(0).unwrap(), PageType::Header);
        assert_eq!(PageType::from_u8(1).unwrap(), PageType::Leaf);
        assert_eq!(PageType::from_u8(2).unwrap(), PageType::Interior);
        assert_eq!(PageType::from_u8(3).unwrap(), PageType::Overflow);
        assert_eq!(PageType::from_u8(4).unwrap(), PageType::Free);
        assert!(PageType::from_u8(5).is_err());
    }
    
    #[test]
    fn test_page_header_serialization() {
        let header = PageHeader {
            page_type: PageType::Leaf,
            cell_count: 10,
            cell_content_offset: 4096,
            fragmented_free_bytes: 128,
            right_child: 0,
        };
        
        let mut bytes = header.to_bytes();
        
        // Pad to minimum expected size
        bytes.resize(PageHeader::SIZE, 0);
        
        let decoded = PageHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.page_type, header.page_type);
        assert_eq!(decoded.cell_count, header.cell_count);
        assert_eq!(decoded.cell_content_offset, header.cell_content_offset);
        assert_eq!(decoded.fragmented_free_bytes, header.fragmented_free_bytes);
    }
    
    #[test]
    fn test_page_initialization() {
        let mut page = Page::new(1, 4096);
        page.initialize(PageType::Leaf, 4096).unwrap();
        
        let header = page.header().unwrap();
        assert_eq!(header.page_type, PageType::Leaf);
        assert_eq!(header.cell_count, 0);
        assert_eq!(header.cell_content_offset, 4096);
    }
}

