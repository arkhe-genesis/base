//! EpisodicSync — CRDT-lite com vector clock
//! Selo: CATHEDRAL-ARKHE-EPISODIC-v1.0.0-2026-06-19

mod sync;
mod storage;
mod types;
mod sqlite_storage;

pub use sync::EpisodicSync;
pub use storage::{JsonlStorage, Storage};
pub use sqlite_storage::SqliteStorage;
pub use types::{EpisodicEntry, VectorClock, Ordering};
