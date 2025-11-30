/// Catalog Manager
/// 
/// Manages catalog persistence and loading

use crate::catalog::schema::*;
use crate::error::{Error, Result};
use crate::storage::pager::Pager;
use crate::planner::logical::{LogicalPlan, DataType as PlanDataType};

/// Special page IDs for system catalog
#[allow(dead_code)]
const CATALOG_ROOT_PAGE: u32 = 1;
#[allow(dead_code)]
const CATALOG_META_KEY: &[u8] = b"__catalog_meta__";

/// Catalog manager - handles catalog persistence
pub struct CatalogManager {
    /// In-memory catalog
    catalog: Catalog,
    
    /// Whether catalog has been modified
    dirty: bool,
}

impl CatalogManager {
    /// Create a new catalog manager
    pub fn new() -> Self {
        CatalogManager {
            catalog: Catalog::new(),
            dirty: false,
        }
    }
    
    /// Load catalog from database
    pub fn load(&mut self, _pager: &mut Pager) -> Result<()> {
        // Try to read catalog from special meta page
        // For now, start with empty catalog
        // TODO: Implement actual persistence using a special B+Tree
        
        self.catalog = Catalog::new();
        self.dirty = false;
        
        Ok(())
    }
    
    /// Save catalog to database
    pub fn save(&mut self, _pager: &mut Pager) -> Result<()> {
        if !self.dirty {
            return Ok(()); // No changes to save
        }
        
        // Serialize catalog
        let _catalog_json = serde_json::to_string(&self.catalog)
            .map_err(|e| Error::Internal(format!("Failed to serialize catalog: {}", e)))?;
        
        // TODO: Write to special meta B+Tree
        // For now, we'll just mark as clean
        
        self.dirty = false;
        Ok(())
    }
    
    /// Get the catalog
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    
    /// Get mutable catalog
    pub fn catalog_mut(&mut self) -> &mut Catalog {
        self.dirty = true;
        &mut self.catalog
    }
    
    /// Execute CREATE TABLE statement
    pub fn create_table(&mut self, plan: &LogicalPlan, pager: &mut Pager) -> Result<()> {
        match plan {
            LogicalPlan::CreateTable { table, columns } => {
                // Check if table already exists
                if self.catalog.get_table(table).is_some() {
                    return Err(Error::Internal(format!("Table '{}' already exists", table)));
                }
                
                // Allocate and initialize a new B+Tree root page
                let root_page = self.initialize_table_btree(pager)?;
                
                // Create table schema
                let mut table_schema = TableSchema::new(table.clone(), root_page);
                
                // Add columns
                for col_spec in columns {
                    let data_type = match col_spec.data_type {
                        PlanDataType::Integer => ColumnType::Integer,
                        PlanDataType::Real => ColumnType::Real,
                        PlanDataType::Text => ColumnType::Text,
                        PlanDataType::Blob => ColumnType::Blob,
                    };
                    
                    let mut column = ColumnSchema::new(col_spec.name.clone(), data_type);
                    
                    if col_spec.primary_key {
                        column = column.with_primary_key();
                    } else if !col_spec.not_null {
                        column = column.with_not_null();
                    }
                    
                    if col_spec.unique {
                        column = column.with_unique();
                    }
                    
                    if let Some(ref default) = col_spec.default {
                        column = column.with_default(default.clone());
                    }
                    
                    table_schema.add_column(column);
                }
                
                // Add to catalog
                self.catalog.add_table(table_schema);
                self.dirty = true;
                
                // Save catalog
                self.save(pager)?;
                
                Ok(())
            }
            _ => Err(Error::Internal("Expected CREATE TABLE plan".to_string())),
        }
    }
    
    /// Allocate a new page for a table
    fn allocate_table_page(&self, pager: &mut Pager) -> Result<u32> {
        // Get the next available page ID
        // For now, use a simple counter based on file size
        let page_id = (pager.file_size()? / pager.page_size() as u64) as u32;
        Ok(page_id)
    }
    
    /// Initialize a B+Tree root page for a table and return its page_id
    fn initialize_table_btree(&self, pager: &mut Pager) -> Result<u32> {
        use crate::storage::page::{PageType};
        
        // Allocate and initialize a leaf page for the table root
        let page = pager.allocate_page(PageType::Leaf)?;
        let page_id = page.id;
        
        // Write the page to ensure it's persisted
        pager.write_page(page)?;
        
        Ok(page_id)
    }
    
    /// Get table schema
    pub fn get_table(&self, name: &str) -> Option<&TableSchema> {
        self.catalog.get_table(name)
    }
    
    /// List all tables
    pub fn list_tables(&self) -> Vec<String> {
        self.catalog.table_names()
    }
    
    /// Drop a table
    pub fn drop_table(&mut self, name: &str) -> Result<()> {
        if self.catalog.remove_table(name).is_none() {
            return Err(Error::Internal(format!("Table '{}' does not exist", name)));
        }
        
        self.dirty = true;
        Ok(())
    }
}

impl Default for CatalogManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::logical::{ColumnSpec, DataType as PlanDataType};
    
    #[test]
    fn test_catalog_manager_creation() {
        let manager = CatalogManager::new();
        assert_eq!(manager.list_tables().len(), 0);
    }
    
    #[test]
    fn test_create_table() {
        let mut manager = CatalogManager::new();
        let mut pager = Pager::open("test_catalog.db").unwrap();
        
        // Create a simple table plan
        let columns = vec![
            ColumnSpec {
                name: "id".to_string(),
                data_type: PlanDataType::Integer,
                not_null: true,
                primary_key: true,
                unique: false,
                default: None,
            },
            ColumnSpec {
                name: "name".to_string(),
                data_type: PlanDataType::Text,
                not_null: true,
                primary_key: false,
                unique: false,
                default: None,
            },
        ];
        
        let plan = LogicalPlan::CreateTable {
            table: "users".to_string(),
            columns,
        };
        
        manager.create_table(&plan, &mut pager).unwrap();
        
        assert_eq!(manager.list_tables().len(), 1);
        assert!(manager.get_table("users").is_some());
        
        let table = manager.get_table("users").unwrap();
        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.primary_key, Some(0));
        
        // Cleanup
        std::fs::remove_file("test_catalog.db").ok();
    }
    
    #[test]
    fn test_duplicate_table() {
        let mut manager = CatalogManager::new();
        let mut pager = Pager::open("test_catalog2.db").unwrap();
        
        let columns = vec![
            ColumnSpec {
                name: "id".to_string(),
                data_type: PlanDataType::Integer,
                not_null: true,
                primary_key: true,
                unique: false,
                default: None,
            },
        ];
        
        let plan = LogicalPlan::CreateTable {
            table: "users".to_string(),
            columns: columns.clone(),
        };
        
        manager.create_table(&plan, &mut pager).unwrap();
        
        // Try to create same table again
        let plan2 = LogicalPlan::CreateTable {
            table: "users".to_string(),
            columns,
        };
        
        let result = manager.create_table(&plan2, &mut pager);
        assert!(result.is_err());
        
        // Cleanup
        std::fs::remove_file("test_catalog2.db").ok();
    }
}

