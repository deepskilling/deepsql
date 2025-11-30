/// File-based locking for multi-reader, single-writer concurrency
/// 
/// Uses file locks to coordinate access between multiple processes.
/// Implements a readers-writer lock pattern:
/// - Multiple readers can access simultaneously
/// - Only one writer at a time
/// - Writers exclude readers and other writers

use crate::error::{Error, Result};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};


/// Lock modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    /// No lock held
    None,
    
    /// Shared lock (read)
    Shared,
    
    /// Exclusive lock (write)
    Exclusive,
}

/// Database lock manager
pub struct LockManager {
    /// Lock file handle
    lock_file: Option<File>,
    
    /// Path to lock file
    #[allow(dead_code)]
    lock_path: PathBuf,
    
    /// Current lock mode
    mode: LockMode,
}

impl LockManager {
    /// Create a new lock manager
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();
        let mut lock_path = db_path.to_path_buf();
        lock_path.set_extension("db-lock");
        
        Ok(LockManager {
            lock_file: None,
            lock_path,
            mode: LockMode::None,
        })
    }
    
    /// Acquire a shared (read) lock
    pub fn lock_shared(&mut self) -> Result<()> {
        if self.mode != LockMode::None {
            return Err(Error::Internal("Lock already held".to_string()));
        }
        
        // Create/open lock file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_path)?;
        
        // Try to acquire shared lock
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let ret = unsafe { libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB) };
            if ret != 0 {
                return Err(Error::Internal("Could not acquire shared lock".to_string()));
            }
        }
        
        #[cfg(windows)]
        {
            // Windows file locking implementation would go here
            // For now, we'll just store the file handle
        }
        
        self.lock_file = Some(file);
        self.mode = LockMode::Shared;
        Ok(())
    }
    
    /// Acquire an exclusive (write) lock
    pub fn lock_exclusive(&mut self) -> Result<()> {
        if self.mode == LockMode::Exclusive {
            return Ok(()); // Already have exclusive lock
        }
        
        // If we have a shared lock, upgrade it
        if self.mode == LockMode::Shared {
            return self.upgrade_lock();
        }
        
        // Create/open lock file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_path)?;
        
        // Try to acquire exclusive lock
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
            if ret != 0 {
                return Err(Error::Internal("Could not acquire exclusive lock".to_string()));
            }
        }
        
        #[cfg(windows)]
        {
            // Windows file locking implementation would go here
        }
        
        self.lock_file = Some(file);
        self.mode = LockMode::Exclusive;
        Ok(())
    }
    
    /// Upgrade a shared lock to exclusive
    fn upgrade_lock(&mut self) -> Result<()> {
        if self.mode != LockMode::Shared {
            return Err(Error::Internal("Can only upgrade from shared lock".to_string()));
        }
        
        #[cfg(unix)]
        {
            if let Some(ref file) = self.lock_file {
                use std::os::unix::io::AsRawFd;
                let fd = file.as_raw_fd();
                
                // Release shared lock
                unsafe { libc::flock(fd, libc::LOCK_UN) };
                
                // Acquire exclusive lock
                let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
                if ret != 0 {
                    // Try to reacquire shared lock
                    unsafe { libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB) };
                    return Err(Error::Internal("Could not upgrade lock".to_string()));
                }
                
                self.mode = LockMode::Exclusive;
                return Ok(());
            }
        }
        
        Err(Error::Internal("No lock file to upgrade".to_string()))
    }
    
    /// Release the current lock
    pub fn unlock(&mut self) -> Result<()> {
        if self.mode == LockMode::None {
            return Ok(());
        }
        
        #[cfg(unix)]
        {
            if let Some(ref file) = self.lock_file {
                use std::os::unix::io::AsRawFd;
                let fd = file.as_raw_fd();
                unsafe { libc::flock(fd, libc::LOCK_UN) };
            }
        }
        
        self.lock_file = None;
        self.mode = LockMode::None;
        Ok(())
    }
    
    /// Get current lock mode
    pub fn mode(&self) -> LockMode {
        self.mode
    }
    
    /// Check if we have a write lock
    pub fn is_write_locked(&self) -> bool {
        self.mode == LockMode::Exclusive
    }
    
    /// Check if we have a read lock
    pub fn is_read_locked(&self) -> bool {
        self.mode == LockMode::Shared || self.mode == LockMode::Exclusive
    }
}

impl Drop for LockManager {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_shared_lock() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut lock = LockManager::new(temp_file.path()).unwrap();
        
        lock.lock_shared().unwrap();
        assert_eq!(lock.mode(), LockMode::Shared);
        assert!(lock.is_read_locked());
        
        lock.unlock().unwrap();
        assert_eq!(lock.mode(), LockMode::None);
    }
    
    #[test]
    fn test_exclusive_lock() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut lock = LockManager::new(temp_file.path()).unwrap();
        
        lock.lock_exclusive().unwrap();
        assert_eq!(lock.mode(), LockMode::Exclusive);
        assert!(lock.is_write_locked());
        
        lock.unlock().unwrap();
        assert_eq!(lock.mode(), LockMode::None);
    }
    
    #[test]
    #[cfg(unix)]
    fn test_lock_upgrade() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut lock = LockManager::new(temp_file.path()).unwrap();
        
        lock.lock_shared().unwrap();
        assert_eq!(lock.mode(), LockMode::Shared);
        
        lock.lock_exclusive().unwrap();
        assert_eq!(lock.mode(), LockMode::Exclusive);
        
        lock.unlock().unwrap();
    }
}

