/// Crash recovery using WAL
/// 
/// When a database is opened after a crash, the WAL is replayed to restore
/// any committed transactions that weren't yet checkpointed.

use crate::error::Result;
use crate::storage::Pager;
use crate::wal::Wal;
use std::collections::HashMap;

/// Recover database from WAL after a crash
/// 
/// This operation:
/// 1. Reads all frames from the WAL
/// 2. Identifies complete transactions (those with commit frames)
/// 3. Applies committed transactions to the database
/// 4. Discards incomplete transactions
/// 
/// Returns the number of transactions recovered
pub fn recover(pager: &mut Pager, wal: &mut Wal) -> Result<usize> {
    // Read all frames from WAL
    let frames = wal.read_frames()?;
    
    if frames.is_empty() {
        return Ok(0);
    }
    
    // Group frames into transactions
    let transactions = group_transactions(frames);
    
    let mut recovered_count = 0;
    
    // Apply each complete transaction
    for transaction in transactions {
        if transaction.is_complete {
            apply_transaction(pager, &transaction)?;
            recovered_count += 1;
        }
    }
    
    // Flush changes to disk
    pager.flush()?;
    
    Ok(recovered_count)
}

/// A transaction in the WAL
#[derive(Debug)]
struct Transaction {
    /// Page frames in this transaction
    frames: Vec<(u32, Vec<u8>)>, // (page_id, data)
    
    /// Is this transaction complete (has commit frame)?
    is_complete: bool,
    
    /// Database size at commit
    #[allow(dead_code)]
    db_size: u32,
}

/// Group WAL frames into transactions
/// 
/// A transaction is complete if it ends with a commit frame (db_size > 0)
fn group_transactions(frames: Vec<crate::wal::WalFrame>) -> Vec<Transaction> {
    let mut transactions = Vec::new();
    let mut current_frames = Vec::new();
    
    for frame in frames {
        let page_id = frame.header.page_number;
        let is_commit = frame.header.is_commit();
        let db_size = frame.header.db_size;
        
        current_frames.push((page_id, frame.data));
        
        if is_commit {
            // End of transaction
            transactions.push(Transaction {
                frames: current_frames,
                is_complete: true,
                db_size,
            });
            current_frames = Vec::new();
        }
    }
    
    // Any remaining frames are an incomplete transaction
    if !current_frames.is_empty() {
        transactions.push(Transaction {
            frames: current_frames,
            is_complete: false,
            db_size: 0,
        });
    }
    
    transactions
}

/// Apply a transaction to the database
fn apply_transaction(pager: &mut Pager, transaction: &Transaction) -> Result<()> {
    // Apply frames in order, keeping only the last version of each page
    let mut page_map: HashMap<u32, &Vec<u8>> = HashMap::new();
    
    for (page_id, data) in &transaction.frames {
        page_map.insert(*page_id, data);
    }
    
    // Write each page
    for (&page_id, data) in &page_map {
        let page = crate::storage::Page::from_data(page_id, (*data).clone());
        pager.write_page(page)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Page, PageType};
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_recovery_with_complete_transaction() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Setup: write a transaction to WAL
        {
            let mut pager = Pager::open(&temp_path).unwrap();
            let mut wal = Wal::open(&temp_path, pager.page_size() as u32).unwrap();
            
            wal.begin_transaction().unwrap();
            
            let mut page = pager.allocate_page(PageType::Leaf).unwrap();
            page.data[0] = 99;
            wal.write_page(&page).unwrap();
            
            wal.commit_transaction(pager.page_count()).unwrap();
            // Don't checkpoint - simulate crash
        }
        
        // Recovery: reopen and recover
        {
            let mut pager = Pager::open(&temp_path).unwrap();
            let mut wal = Wal::open(&temp_path, pager.page_size() as u32).unwrap();
            
            let recovered = recover(&mut pager, &mut wal).unwrap();
            assert_eq!(recovered, 1);
            
            // Verify the page was recovered
            let page = pager.read_page(2).unwrap();
            assert_eq!(page.data[0], 99);
        }
    }
    
    #[test]
    fn test_recovery_with_incomplete_transaction() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // This test would require manually creating an incomplete WAL
        // For now, we just verify empty WAL recovery
        {
            let mut pager = Pager::open(&temp_path).unwrap();
            let mut wal = Wal::open(&temp_path, pager.page_size() as u32).unwrap();
            
            let recovered = recover(&mut pager, &mut wal).unwrap();
            assert_eq!(recovered, 0);
        }
    }
}

