//! EpisodicSync — CRDT-lite com vector clock
//! Selo: CATHEDRAL-ARKHE-EPISODIC-v1.0.0-2026-06-19

mod sqlite_storage;
mod storage;
mod sync;
mod types;

pub use sqlite_storage::SqliteStorage;
pub use storage::{JsonlStorage, Storage};
pub use sync::EpisodicSync;
pub use types::{EpisodicEntry, Ordering, VectorClock};
