/// WAL frame format
/// 
/// Each WAL frame contains:
/// - Frame header (page number, checksum, commit flag)
/// - Page data (full page content)

use crate::error::{Error, Result};
use crate::storage::PageId;

/// WAL magic number: "WALv1\0\0\0"
pub const WAL_MAGIC: [u8; 8] = [b'W', b'A', b'L', b'v', b'1', 0, 0, 0];

/// WAL header (at start of WAL file)
#[derive(Debug, Clone)]
pub struct WalHeader {
    /// Magic bytes for WAL identification
    pub magic: [u8; 8],
    
    /// WAL format version
    pub version: u32,
    
    /// Page size (must match database)
    pub page_size: u32,
    
    /// Checkpoint sequence number
    pub checkpoint_seq: u32,
    
    /// Salt-1: changes on each checkpoint
    pub salt_1: u32,
    
    /// Salt-2: random value set at WAL creation
    pub salt_2: u32,
    
    /// Checksum of header
    pub checksum: u32,
}

impl WalHeader {
    /// Size of WAL header in bytes
    pub const SIZE: usize = 32;
    
    /// Create a new WAL header
    pub fn new(page_size: u32) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let salt_2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        
        WalHeader {
            magic: WAL_MAGIC,
            version: 1,
            page_size,
            checkpoint_seq: 0,
            salt_1: 0,
            salt_2,
            checksum: 0,
        }
    }
    
    /// Serialize header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.extend_from_slice(&self.page_size.to_be_bytes());
        bytes.extend_from_slice(&self.checkpoint_seq.to_be_bytes());
        bytes.extend_from_slice(&self.salt_1.to_be_bytes());
        bytes.extend_from_slice(&self.salt_2.to_be_bytes());
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        
        bytes
    }
    
    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(Error::Corruption("WAL header too short".to_string()));
        }
        
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        
        if magic != WAL_MAGIC {
            return Err(Error::Corruption("Invalid WAL magic bytes".to_string()));
        }
        
        let version = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let page_size = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let checkpoint_seq = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let salt_1 = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let salt_2 = u32::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let checksum = u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        
        Ok(WalHeader {
            magic,
            version,
            page_size,
            checkpoint_seq,
            salt_1,
            salt_2,
            checksum,
        })
    }
    
    /// Compute checksum for the header
    pub fn compute_checksum(&mut self) {
        // Simple checksum: XOR of all fields
        let mut checksum: u32 = 0;
        checksum ^= self.version;
        checksum ^= self.page_size;
        checksum ^= self.checkpoint_seq;
        checksum ^= self.salt_1;
        checksum ^= self.salt_2;
        
        self.checksum = checksum;
    }
}

/// WAL frame header
#[derive(Debug, Clone)]
pub struct WalFrameHeader {
    /// Page number in database
    pub page_number: PageId,
    
    /// Database size after commit (0 if not a commit frame)
    pub db_size: u32,
    
    /// Salt-1 from WAL header
    pub salt_1: u32,
    
    /// Salt-2 from WAL header
    pub salt_2: u32,
    
    /// Checksum of frame
    pub checksum: u32,
}

impl WalFrameHeader {
    /// Size of frame header in bytes
    pub const SIZE: usize = 24;
    
    /// Create a new frame header
    pub fn new(page_number: PageId, db_size: u32, salt_1: u32, salt_2: u32) -> Self {
        WalFrameHeader {
            page_number,
            db_size,
            salt_1,
            salt_2,
            checksum: 0,
        }
    }
    
    /// Serialize frame header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        
        bytes.extend_from_slice(&self.page_number.to_be_bytes());
        bytes.extend_from_slice(&self.db_size.to_be_bytes());
        bytes.extend_from_slice(&self.salt_1.to_be_bytes());
        bytes.extend_from_slice(&self.salt_2.to_be_bytes());
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&[0u8; 4]); // Reserved
        
        bytes
    }
    
    /// Deserialize frame header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(Error::Corruption("WAL frame header too short".to_string()));
        }
        
        let page_number = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let db_size = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let salt_1 = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let salt_2 = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let checksum = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        
        Ok(WalFrameHeader {
            page_number,
            db_size,
            salt_1,
            salt_2,
            checksum,
        })
    }
    
    /// Compute checksum for frame (header + page data)
    pub fn compute_checksum(&mut self, page_data: &[u8]) {
        let mut checksum: u32 = 0;
        
        // Checksum frame header fields
        checksum ^= self.page_number;
        checksum ^= self.db_size;
        checksum ^= self.salt_1;
        checksum ^= self.salt_2;
        
        // Checksum page data (sample every 256 bytes for speed)
        for (i, &byte) in page_data.iter().enumerate() {
            if i % 256 == 0 {
                checksum ^= (byte as u32) << ((i % 4) * 8);
            }
        }
        
        self.checksum = checksum;
    }
    
    /// Check if this is a commit frame
    pub fn is_commit(&self) -> bool {
        self.db_size > 0
    }
}

/// Complete WAL frame (header + page data)
#[derive(Debug, Clone)]
pub struct WalFrame {
    /// Frame header
    pub header: WalFrameHeader,
    
    /// Page data
    pub data: Vec<u8>,
}

impl WalFrame {
    /// Create a new WAL frame
    pub fn new(page_number: PageId, data: Vec<u8>, db_size: u32, salt_1: u32, salt_2: u32) -> Self {
        let mut header = WalFrameHeader::new(page_number, db_size, salt_1, salt_2);
        header.compute_checksum(&data);
        
        WalFrame { header, data }
    }
    
    /// Total size of frame (header + data)
    pub fn size(&self) -> usize {
        WalFrameHeader::SIZE + self.data.len()
    }
    
    /// Serialize frame to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.data);
        bytes
    }
    
    /// Deserialize frame from bytes
    pub fn from_bytes(bytes: &[u8], page_size: usize) -> Result<Self> {
        if bytes.len() < WalFrameHeader::SIZE + page_size {
            return Err(Error::Corruption("WAL frame too short".to_string()));
        }
        
        let header = WalFrameHeader::from_bytes(&bytes[0..WalFrameHeader::SIZE])?;
        let data = bytes[WalFrameHeader::SIZE..WalFrameHeader::SIZE + page_size].to_vec();
        
        // Verify checksum
        let mut verify_header = header.clone();
        verify_header.compute_checksum(&data);
        
        if verify_header.checksum != header.checksum {
            return Err(Error::Corruption("WAL frame checksum mismatch".to_string()));
        }
        
        Ok(WalFrame { header, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wal_header_serialization() {
        let mut header = WalHeader::new(4096);
        header.compute_checksum();
        
        let bytes = header.to_bytes();
        let decoded = WalHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.magic, WAL_MAGIC);
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.page_size, 4096);
        assert_eq!(decoded.checksum, header.checksum);
    }
    
    #[test]
    fn test_wal_frame_serialization() {
        let page_data = vec![42u8; 4096];
        let frame = WalFrame::new(1, page_data.clone(), 0, 100, 200);
        
        let bytes = frame.to_bytes();
        let decoded = WalFrame::from_bytes(&bytes, 4096).unwrap();
        
        assert_eq!(decoded.header.page_number, 1);
        assert_eq!(decoded.data, page_data);
    }
    
    #[test]
    fn test_commit_frame() {
        let page_data = vec![42u8; 4096];
        let frame = WalFrame::new(1, page_data, 10, 100, 200);
        
        assert!(frame.header.is_commit());
    }
    
    #[test]
    fn test_non_commit_frame() {
        let page_data = vec![42u8; 4096];
        let frame = WalFrame::new(1, page_data, 0, 100, 200);
        
        assert!(!frame.header.is_commit());
    }
}

