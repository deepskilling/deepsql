/// Page Manager (Pager)
/// 
/// The Pager is responsible for:
/// - Reading and writing pages from/to disk
/// - Managing an in-memory page cache
/// - Allocating new pages
/// - Managing the free page list
/// - Ensuring pages are flushed to disk

use crate::error::{Error, Result};
use crate::storage::file_format::{DatabaseHeader, HEADER_PAGE_ID, DEFAULT_PAGE_SIZE};
use crate::storage::page::{Page, PageId, PageType};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Maximum number of pages to keep in memory cache
const DEFAULT_CACHE_SIZE: usize = 256;

/// Page Manager - handles page I/O and caching
pub struct Pager {
    /// Database file handle
    file: File,
    
    /// Path to the database file
    path: PathBuf,
    
    /// Database header
    header: DatabaseHeader,
    
    /// In-memory page cache
    cache: HashMap<PageId, Page>,
    
    /// Maximum cache size
    cache_size: usize,
    
    /// Page size in bytes
    page_size: usize,
    
    /// Transaction mode flag
    transaction_mode: bool,
    
    /// Shadow pages for transaction (page_id -> original_data)
    shadow_pages: HashMap<PageId, Vec<u8>>,
    
    /// Modified pages in transaction
    modified_pages: HashMap<PageId, Page>,
}

impl Pager {
    /// Open or create a database file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_cache_size(path, DEFAULT_CACHE_SIZE)
    }
    
    /// Open or create a database file with custom cache size
    pub fn open_with_cache_size<P: AsRef<Path>>(path: P, cache_size: usize) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        
        // Check file size to determine if it's a new or existing database
        let file_size = file.metadata()?.len();
        let file_exists = file_size >= DEFAULT_PAGE_SIZE as u64;
        
        let (header, page_size) = if file_exists {
            // Read existing header
            let mut header_data = vec![0u8; DEFAULT_PAGE_SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut header_data)?;
            
            let header = DatabaseHeader::from_bytes(&header_data)?;
            let page_size = header.page_size as usize;
            
            (header, page_size)
        } else {
            // Create new database
            let header = DatabaseHeader::new(DEFAULT_PAGE_SIZE as u32)?;
            let page_size = header.page_size as usize;
            
            // Write header page
            let mut header_page = vec![0u8; page_size];
            let header_bytes = header.to_bytes();
            header_page[..header_bytes.len()].copy_from_slice(&header_bytes);
            
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&header_page)?;
            file.sync_all()?;
            
            (header, page_size)
        };
        
        Ok(Pager {
            file,
            path,
            header,
            cache: HashMap::new(),
            cache_size,
            page_size,
            transaction_mode: false,
            shadow_pages: HashMap::new(),
            modified_pages: HashMap::new(),
        })
    }
    
    /// Enable transaction mode
    pub fn begin_transaction_mode(&mut self) {
        self.transaction_mode = true;
        self.shadow_pages.clear();
        self.modified_pages.clear();
    }
    
    /// Disable transaction mode and clear state
    pub fn end_transaction_mode(&mut self) {
        self.transaction_mode = false;
        self.shadow_pages.clear();
        self.modified_pages.clear();
    }
    
    /// Get modified pages from transaction
    pub fn modified_pages(&self) -> &HashMap<PageId, Page> {
        &self.modified_pages
    }
    
    /// Get shadow pages for rollback
    pub fn shadow_pages(&self) -> &HashMap<PageId, Vec<u8>> {
        &self.shadow_pages
    }
    
    /// Get database header
    pub fn header(&self) -> &DatabaseHeader {
        &self.header
    }
    
    /// Get page size
    pub fn page_size(&self) -> usize {
        self.page_size
    }
    
    /// Read a page from disk or cache
    /// If there's an active transaction with a modified version, returns that instead
    pub fn read_page(&mut self, page_id: PageId) -> Result<Page> {
        // Check if page is in cache
        if let Some(page) = self.cache.get(&page_id) {
            return Ok(page.clone());
        }
        
        // Validate page ID
        if page_id == 0 {
            return Err(Error::InvalidPage("Page ID cannot be 0".to_string()));
        }
        
        if page_id > self.header.page_count {
            return Err(Error::InvalidPage(
                format!("Page {} does not exist (max: {})", page_id, self.header.page_count)
            ));
        }
        
        // Read from disk
        let offset = (page_id - 1) as u64 * self.page_size as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        
        let mut data = vec![0u8; self.page_size];
        self.file.read_exact(&mut data)?;
        
        let page = Page::from_data(page_id, data);
        
        // Add to cache
        self.add_to_cache(page.clone());
        
        Ok(page)
    }
    
    /// Write a page (transaction-aware)
    pub fn write_page(&mut self, page: Page) -> Result<()> {
        let page_id = page.id;
        
        // Validate page ID
        if page_id == 0 {
            return Err(Error::InvalidPage("Page ID cannot be 0".to_string()));
        }
        
        if self.transaction_mode {
            // In transaction mode: save shadow copy and track modification
            if !self.shadow_pages.contains_key(&page_id) {
                // Save original page data
                let original = if let Some(cached) = self.cache.get(&page_id) {
                    cached.data.clone()
                } else {
                    // Read from disk
                    let offset = (page_id - 1) as u64 * self.page_size as u64;
                    self.file.seek(SeekFrom::Start(offset))?;
                    let mut data = vec![0u8; self.page_size];
                    self.file.read_exact(&mut data)?;
                    data
                };
                self.shadow_pages.insert(page_id, original);
            }
            
            // Track modified page (don't write to disk yet)
            self.modified_pages.insert(page_id, page.clone());
            self.add_to_cache(page);
        } else {
            // Normal mode: write directly to disk
            let offset = (page_id - 1) as u64 * self.page_size as u64;
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(&page.data)?;
            
            let mut cached_page = page;
            cached_page.dirty = false;
            self.add_to_cache(cached_page);
        }
        
        Ok(())
    }
    
    /// Commit transaction: write all modified pages to disk
    pub fn commit_transaction_pages(&mut self) -> Result<()> {
        let pages: Vec<_> = self.modified_pages.values().cloned().collect();
        
        for page in pages {
            let page_id = page.id;
            let offset = (page_id - 1) as u64 * self.page_size as u64;
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(&page.data)?;
            
            let mut cached_page = page;
            cached_page.dirty = false;
            self.add_to_cache(cached_page);
        }
        Ok(())
    }
    
    /// Rollback transaction: restore original pages
    pub fn rollback_transaction_pages(&mut self) -> Result<()> {
        let shadow_copies: Vec<_> = self.shadow_pages.iter()
            .map(|(id, data)| (*id, data.clone()))
            .collect();
        
        for (page_id, original_data) in shadow_copies {
            let page = Page::from_data(page_id, original_data.clone());
            self.add_to_cache(page);
            
            // Write back to disk
            let offset = (page_id - 1) as u64 * self.page_size as u64;
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(&original_data)?;
        }
        Ok(())
    }
    
    /// Allocate a new page
    pub fn allocate_page(&mut self, page_type: PageType) -> Result<Page> {
        // TODO: Use free list if available
        
        // Allocate new page at end of file
        let page_id = self.header.page_count + 1;
        let mut page = Page::new(page_id, self.page_size);
        page.initialize(page_type, self.page_size)?;
        
        // Update header
        self.header.page_count = page_id;
        self.write_header()?;
        
        // Write the new page
        self.write_page(page.clone())?;
        
        Ok(page)
    }
    
    /// Free a page (add to free list)
    pub fn free_page(&mut self, page_id: PageId) -> Result<()> {
        if page_id == HEADER_PAGE_ID {
            return Err(Error::InvalidArgument("Cannot free header page".to_string()));
        }
        
        // Read the page
        let mut page = self.read_page(page_id)?;
        
        // Mark as free and link to free list
        page.initialize(PageType::Free, self.page_size)?;
        
        // Store next free page pointer in the page data
        let next_free = self.header.first_free_page;
        page.data[0..4].copy_from_slice(&next_free.to_be_bytes());
        
        // Write the page
        self.write_page(page)?;
        
        // Update header to point to this page as first free
        self.header.first_free_page = page_id;
        self.write_header()?;
        
        Ok(())
    }
    
    /// Flush all dirty pages to disk
    pub fn flush(&mut self) -> Result<()> {
        let dirty_pages: Vec<Page> = self.cache
            .values()
            .filter(|p| p.dirty)
            .cloned()
            .collect();
        
        for page in dirty_pages {
            self.write_page(page)?;
        }
        
        self.file.sync_all()?;
        Ok(())
    }
    
    /// Write header to disk
    fn write_header(&mut self) -> Result<()> {
        let mut header_page = vec![0u8; self.page_size];
        let header_bytes = self.header.to_bytes();
        header_page[..header_bytes.len()].copy_from_slice(&header_bytes);
        
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&header_page)?;
        self.file.sync_all()?;
        
        Ok(())
    }
    
    /// Add page to cache (evict if necessary)
    fn add_to_cache(&mut self, page: Page) {
        // Simple eviction: if cache is full, remove first entry
        // TODO: Implement LRU eviction policy
        if self.cache.len() >= self.cache_size {
            if let Some(key) = self.cache.keys().next().copied() {
                self.cache.remove(&key);
            }
        }
        
        self.cache.insert(page.id, page);
    }
    
    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get number of pages in database
    pub fn page_count(&self) -> u32 {
        self.header.page_count
    }
    
    /// Set root page
    pub fn set_root_page(&mut self, root_page: PageId) -> Result<()> {
        self.header.root_page = root_page;
        self.write_header()
    }
    
    /// Get root page
    pub fn root_page(&self) -> PageId {
        self.header.root_page
    }
    
    /// Get database file path
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
    
    /// Get the size of the database file
    pub fn file_size(&self) -> Result<u64> {
        self.file.metadata()
            .map(|m| m.len())
            .map_err(|e| Error::Io(format!("Failed to get file size: {}", e)))
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        // Best effort flush on drop
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_create_database() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        let pager = Pager::open(path).unwrap();
        assert_eq!(pager.page_size(), DEFAULT_PAGE_SIZE);
        assert_eq!(pager.page_count(), 1);
    }
    
    #[test]
    fn test_allocate_page() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        let mut pager = Pager::open(path).unwrap();
        let page = pager.allocate_page(PageType::Leaf).unwrap();
        
        assert_eq!(page.id, 2);
        assert_eq!(pager.page_count(), 2);
    }
    
    #[test]
    fn test_read_write_page() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        let mut pager = Pager::open(path).unwrap();
        let mut page = pager.allocate_page(PageType::Leaf).unwrap();
        
        // Modify page data
        page.data[100] = 42;
        pager.write_page(page.clone()).unwrap();
        
        // Clear cache and read again
        pager.clear_cache();
        let read_page = pager.read_page(page.id).unwrap();
        
        assert_eq!(read_page.data[100], 42);
    }
    
    #[test]
    fn test_reopen_database() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        
        // Create and write
        {
            let mut pager = Pager::open(&path).unwrap();
            pager.allocate_page(PageType::Leaf).unwrap();
            pager.flush().unwrap();
        }
        
        // Reopen and verify
        {
            let pager = Pager::open(&path).unwrap();
            assert_eq!(pager.page_count(), 2);
        }
    }
}

