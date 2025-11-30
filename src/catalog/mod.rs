/// Catalog & Schema Management
/// 
/// Manages database metadata (tables, columns, indexes)

/// Catalog structures
pub mod schema;

/// Catalog manager
pub mod manager;

pub use schema::*;
pub use manager::CatalogManager;

