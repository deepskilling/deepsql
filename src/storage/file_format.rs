/// Database file format constants and utilities
/// 
/// DeepSQL uses a single-file database format similar to SQLite.
/// The file consists of fixed-size pages, with the first page being a special header page.

use crate::error::{Error, Result};

/// Magic number identifying a DeepSQL database file: "DSQLv1\0\0"
pub const MAGIC_BYTES: [u8; 8] = [b'D', b'S', b'Q', b'L', b'v', b'1', 0, 0];

/// Database file format version
pub const FORMAT_VERSION: u32 = 1;

/// Default page size (4KB)
pub const DEFAULT_PAGE_SIZE: usize = 4096;

/// Minimum page size (512 bytes)
pub const MIN_PAGE_SIZE: usize = 512;

/// Maximum page size (64KB)
pub const MAX_PAGE_SIZE: usize = 65536;

/// Header page is always page 1 (first page in file)
pub const HEADER_PAGE_ID: u32 = 1;

/// Database header structure stored in the first page
#[derive(Debug, Clone)]
pub struct DatabaseHeader {
    /// Magic bytes for file identification
    pub magic: [u8; 8],
    
    /// File format version
    pub version: u32,
    
    /// Page size in bytes (must be power of 2)
    pub page_size: u32,
    
    /// Total number of pages in the database
    pub page_count: u32,
    
    /// First free page (freelist head)
    pub first_free_page: u32,
    
    /// Root page of the main table B+Tree
    pub root_page: u32,
    
    /// Schema version (incremented on schema changes)
    pub schema_version: u32,
    
    /// Reserved for future use
    pub reserved: [u8; 32],
}

impl DatabaseHeader {
    /// Size of the header in bytes
    pub const SIZE: usize = 96;
    
    /// Create a new database header with default values
    pub fn new(page_size: u32) -> Result<Self> {
        if !page_size.is_power_of_two() {
            return Err(Error::InvalidArgument(
                "Page size must be a power of 2".to_string()
            ));
        }
        
        if (page_size as usize) < MIN_PAGE_SIZE || (page_size as usize) > MAX_PAGE_SIZE {
            return Err(Error::InvalidArgument(
                format!("Page size must be between {} and {} bytes", 
                        MIN_PAGE_SIZE, MAX_PAGE_SIZE)
            ));
        }
        
        Ok(DatabaseHeader {
            magic: MAGIC_BYTES,
            version: FORMAT_VERSION,
            page_size,
            page_count: 1, // Start with just the header page
            first_free_page: 0, // No free pages initially
            root_page: 0, // No root page yet
            schema_version: 0,
            reserved: [0; 32],
        })
    }
    
    /// Serialize header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        
        // Magic bytes
        bytes.extend_from_slice(&self.magic);
        
        // Version
        bytes.extend_from_slice(&self.version.to_be_bytes());
        
        // Page size
        bytes.extend_from_slice(&self.page_size.to_be_bytes());
        
        // Page count
        bytes.extend_from_slice(&self.page_count.to_be_bytes());
        
        // First free page
        bytes.extend_from_slice(&self.first_free_page.to_be_bytes());
        
        // Root page
        bytes.extend_from_slice(&self.root_page.to_be_bytes());
        
        // Schema version
        bytes.extend_from_slice(&self.schema_version.to_be_bytes());
        
        // Reserved
        bytes.extend_from_slice(&self.reserved);
        
        bytes
    }
    
    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(Error::Corruption(
                "Header too short".to_string()
            ));
        }
        
        // Parse magic bytes
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        
        if magic != MAGIC_BYTES {
            return Err(Error::Corruption(
                "Invalid magic bytes".to_string()
            ));
        }
        
        // Parse version
        let version = u32::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11]
        ]);
        
        if version != FORMAT_VERSION {
            return Err(Error::Corruption(
                format!("Unsupported format version: {}", version)
            ));
        }
        
        // Parse page size
        let page_size = u32::from_be_bytes([
            bytes[12], bytes[13], bytes[14], bytes[15]
        ]);
        
        // Parse page count
        let page_count = u32::from_be_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19]
        ]);
        
        // Parse first free page
        let first_free_page = u32::from_be_bytes([
            bytes[20], bytes[21], bytes[22], bytes[23]
        ]);
        
        // Parse root page
        let root_page = u32::from_be_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27]
        ]);
        
        // Parse schema version
        let schema_version = u32::from_be_bytes([
            bytes[28], bytes[29], bytes[30], bytes[31]
        ]);
        
        // Parse reserved
        let mut reserved = [0u8; 32];
        reserved.copy_from_slice(&bytes[32..64]);
        
        Ok(DatabaseHeader {
            magic,
            version,
            page_size,
            page_count,
            first_free_page,
            root_page,
            schema_version,
            reserved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_header_creation() {
        let header = DatabaseHeader::new(4096).unwrap();
        assert_eq!(header.magic, MAGIC_BYTES);
        assert_eq!(header.version, FORMAT_VERSION);
        assert_eq!(header.page_size, 4096);
        assert_eq!(header.page_count, 1);
    }
    
    #[test]
    fn test_invalid_page_size() {
        assert!(DatabaseHeader::new(1000).is_err()); // Not power of 2
        assert!(DatabaseHeader::new(256).is_err()); // Too small
        assert!(DatabaseHeader::new(131072).is_err()); // Too large
    }
    
    #[test]
    fn test_header_serialization() {
        let header = DatabaseHeader::new(4096).unwrap();
        let mut bytes = header.to_bytes();
        
        // Pad to minimum size to simulate how it's stored in a page
        bytes.resize(DatabaseHeader::SIZE, 0);
        
        let decoded = DatabaseHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.magic, header.magic);
        assert_eq!(decoded.version, header.version);
        assert_eq!(decoded.page_size, header.page_size);
        assert_eq!(decoded.page_count, header.page_count);
    }
}

