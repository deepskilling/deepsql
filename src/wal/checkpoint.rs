/// WAL checkpoint mechanism
/// 
/// Checkpointing copies all frames from the WAL back to the main database file
/// and then truncates the WAL. This ensures the WAL doesn't grow indefinitely.

use crate::error::Result;
use crate::storage::Pager;
use crate::wal::Wal;

/// Perform a checkpoint: copy WAL frames to main database
/// 
/// This operation:
/// 1. Reads all frames from the WAL
/// 2. Writes them to the main database file
/// 3. Syncs the database file
/// 4. Truncates the WAL
pub fn checkpoint(pager: &mut Pager, wal: &mut Wal) -> Result<usize> {
    // Read all frames from WAL
    let frames = wal.read_frames()?;
    
    if frames.is_empty() {
        return Ok(0);
    }
    
    let mut pages_written = 0;
    
    // Write each frame to the database
    for frame in frames {
        let page_id = frame.header.page_number;
        let page = crate::storage::Page::from_data(page_id, frame.data);
        
        pager.write_page(page)?;
        pages_written += 1;
    }
    
    // Flush pager to ensure all changes are on disk
    pager.flush()?;
    
    // Truncate the WAL
    wal.truncate()?;
    
    Ok(pages_written)
}

/// Perform a passive checkpoint (non-blocking)
/// 
/// Similar to regular checkpoint but doesn't wait for locks
pub fn checkpoint_passive(pager: &mut Pager, wal: &mut Wal) -> Result<usize> {
    // For Phase 2, passive is the same as regular
    // In Phase 3+, this would check for locks
    checkpoint(pager, wal)
}

/// Perform a full checkpoint (wait for all operations)
/// 
/// This ensures all WAL content is written to database
pub fn checkpoint_full(pager: &mut Pager, wal: &mut Wal) -> Result<usize> {
    checkpoint(pager, wal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Page;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_checkpoint() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut pager = Pager::open(&temp_path).unwrap();
        let mut wal = Wal::open(&temp_path, pager.page_size() as u32).unwrap();
        
        // Write some frames to WAL
        wal.begin_transaction().unwrap();
        
        let mut page = pager.allocate_page(crate::storage::PageType::Leaf).unwrap();
        page.data[0] = 42;
        wal.write_page(&page).unwrap();
        
        wal.commit_transaction(pager.page_count()).unwrap();
        
        // Perform checkpoint
        let pages_written = checkpoint(&mut pager, &mut wal).unwrap();
        assert_eq!(pages_written, 1);
        
        // Verify WAL is truncated
        assert_eq!(wal.frame_count(), 0);
    }
}

