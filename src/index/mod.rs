/// Index Support
/// 
/// Implements secondary indexes for improved query performance

/// Index manager
pub mod manager;

/// Index B+Tree implementation
pub mod index_btree;

pub use manager::IndexManager;
pub use index_btree::IndexBTree;

