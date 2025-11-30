/// Write-Ahead Log (WAL) implementation
/// 
/// The WAL provides ACID transaction support and crash recovery.
/// It logs all changes before they're written to the main database file.

/// Frame format and structures
pub mod frame;
/// WAL file manager
pub mod wal;
/// Checkpoint operations
pub mod checkpoint;
/// Crash recovery
pub mod recovery;

pub use frame::{WalFrame, WalHeader};
pub use wal::Wal;
pub use checkpoint::checkpoint;
pub use recovery::recover;

