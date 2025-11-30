/// WAL (Write-Ahead Log) manager
/// 
/// Handles writing frames to the WAL file and managing transactions

use crate::error::{Error, Result};
use crate::storage::{Page, PageId};
use crate::wal::frame::{WalFrame, WalHeader, WalFrameHeader};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// WAL file manager
pub struct Wal {
    /// WAL file handle
    file: File,
    
    /// WAL file path
    #[allow(dead_code)]
    path: PathBuf,
    
    /// WAL header
    header: WalHeader,
    
    /// Current transaction frames (not yet committed)
    transaction_frames: HashMap<PageId, Vec<u8>>,
    
    /// Is there an active transaction?
    in_transaction: bool,
    
    /// Number of frames written since last checkpoint
    frame_count: u32,
}

impl Wal {
    /// Open or create a WAL file
    pub fn open<P: AsRef<Path>>(db_path: P, page_size: u32) -> Result<Self> {
        let db_path = db_path.as_ref();
        let wal_path = Self::wal_path(db_path);
        
        let file_exists = wal_path.exists();
        
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&wal_path)?;
        
        let header = if file_exists && file.metadata()?.len() >= WalHeader::SIZE as u64 {
            // Read existing header
            let mut header_bytes = vec![0u8; WalHeader::SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut header_bytes)?;
            WalHeader::from_bytes(&header_bytes)?
        } else {
            // Create new header
            let mut header = WalHeader::new(page_size);
            header.compute_checksum();
            
            // Write header
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&header.to_bytes())?;
            file.sync_all()?;
            
            header
        };
        
        Ok(Wal {
            file,
            path: wal_path,
            header,
            transaction_frames: HashMap::new(),
            in_transaction: false,
            frame_count: 0,
        })
    }
    
    /// Get WAL file path from database path
    fn wal_path(db_path: &Path) -> PathBuf {
        let mut wal_path = db_path.to_path_buf();
        wal_path.set_extension("db-wal");
        wal_path
    }
    
    /// Begin a transaction
    pub fn begin_transaction(&mut self) -> Result<()> {
        if self.in_transaction {
            return Err(Error::Internal("Transaction already active".to_string()));
        }
        
        self.transaction_frames.clear();
        self.in_transaction = true;
        Ok(())
    }
    
    /// Write a page to the WAL (buffer in transaction)
    pub fn write_page(&mut self, page: &Page) -> Result<()> {
        if !self.in_transaction {
            return Err(Error::Internal("No active transaction".to_string()));
        }
        
        // Store page data in transaction buffer
        self.transaction_frames.insert(page.id, page.data.clone());
        Ok(())
    }
    
    /// Commit the current transaction
    pub fn commit_transaction(&mut self, db_size: u32) -> Result<()> {
        if !self.in_transaction {
            return Err(Error::Internal("No active transaction".to_string()));
        }
        
        if self.transaction_frames.is_empty() {
            // Nothing to commit
            self.in_transaction = false;
            return Ok(());
        }
        
        // Write all frames to WAL
        self.file.seek(SeekFrom::End(0))?;
        
        // Sort page IDs for consistent ordering
        let mut page_ids: Vec<PageId> = self.transaction_frames.keys().copied().collect();
        page_ids.sort();
        
        // Write all frames except the last one
        for (i, &page_id) in page_ids.iter().enumerate() {
            let page_data = self.transaction_frames.get(&page_id).unwrap();
            let is_commit = i == page_ids.len() - 1;
            let frame_db_size = if is_commit { db_size } else { 0 };
            
            let frame = WalFrame::new(
                page_id,
                page_data.clone(),
                frame_db_size,
                self.header.salt_1,
                self.header.salt_2,
            );
            
            self.file.write_all(&frame.to_bytes())?;
            self.frame_count += 1;
        }
        
        // Sync to disk (durability)
        self.file.sync_all()?;
        
        // Clear transaction
        self.transaction_frames.clear();
        self.in_transaction = false;
        
        Ok(())
    }
    
    /// Rollback the current transaction
    pub fn rollback_transaction(&mut self) -> Result<()> {
        if !self.in_transaction {
            return Err(Error::Internal("No active transaction".to_string()));
        }
        
        self.transaction_frames.clear();
        self.in_transaction = false;
        Ok(())
    }
    
    /// Read all frames from WAL
    pub fn read_frames(&mut self) -> Result<Vec<WalFrame>> {
        let mut frames = Vec::new();
        
        // Seek to start of frames (after header)
        self.file.seek(SeekFrom::Start(WalHeader::SIZE as u64))?;
        
        let page_size = self.header.page_size as usize;
        let frame_size = WalFrameHeader::SIZE + page_size;
        
        loop {
            let mut frame_bytes = vec![0u8; frame_size];
            
            match self.file.read_exact(&mut frame_bytes) {
                Ok(_) => {
                    match WalFrame::from_bytes(&frame_bytes, page_size) {
                        Ok(frame) => frames.push(frame),
                        Err(_) => break, // Corrupted frame, stop reading
                    }
                }
                Err(_) => break, // End of file
            }
        }
        
        Ok(frames)
    }
    
    /// Get the number of frames in the WAL
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
    
    /// Check if checkpoint is needed (too many frames)
    pub fn needs_checkpoint(&self) -> bool {
        self.frame_count > 1000 // Checkpoint after 1000 frames
    }
    
    /// Truncate the WAL (after checkpoint)
    pub fn truncate(&mut self) -> Result<()> {
        // Increment checkpoint sequence
        self.header.checkpoint_seq += 1;
        self.header.salt_1 = self.header.checkpoint_seq;
        self.header.compute_checksum();
        
        // Rewrite header
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&self.header.to_bytes())?;
        
        // Truncate file after header
        self.file.set_len(WalHeader::SIZE as u64)?;
        self.file.sync_all()?;
        
        self.frame_count = 0;
        
        Ok(())
    }
    
    /// Get WAL header
    pub fn header(&self) -> &WalHeader {
        &self.header
    }
    
    /// Check if in transaction
    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }
}

impl Drop for Wal {
    fn drop(&mut self) {
        // If transaction is still active, try to sync
        let _ = self.file.sync_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_wal_create() {
        let temp_file = NamedTempFile::new().unwrap();
        let wal = Wal::open(temp_file.path(), 4096).unwrap();
        
        assert_eq!(wal.header().page_size, 4096);
        assert!(!wal.in_transaction());
    }
    
    #[test]
    fn test_transaction_lifecycle() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut wal = Wal::open(temp_file.path(), 4096).unwrap();
        
        // Begin transaction
        wal.begin_transaction().unwrap();
        assert!(wal.in_transaction());
        
        // Write a page
        let page = Page::new(1, 4096);
        wal.write_page(&page).unwrap();
        
        // Commit
        wal.commit_transaction(1).unwrap();
        assert!(!wal.in_transaction());
    }
    
    #[test]
    fn test_rollback() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut wal = Wal::open(temp_file.path(), 4096).unwrap();
        
        wal.begin_transaction().unwrap();
        let page = Page::new(1, 4096);
        wal.write_page(&page).unwrap();
        
        // Rollback
        wal.rollback_transaction().unwrap();
        assert!(!wal.in_transaction());
    }
    
    #[test]
    fn test_read_frames() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut wal = Wal::open(temp_file.path(), 4096).unwrap();
        
        // Write some frames
        wal.begin_transaction().unwrap();
        
        for i in 1..=3 {
            let mut page = Page::new(i, 4096);
            page.data[0] = i as u8;
            wal.write_page(&page).unwrap();
        }
        
        wal.commit_transaction(3).unwrap();
        
        // Read them back
        let frames = wal.read_frames().unwrap();
        assert_eq!(frames.len(), 3);
        assert!(frames[2].header.is_commit());
    }
}

