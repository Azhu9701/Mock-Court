pub mod config;
pub mod error;
pub mod fs_store;
pub mod health;
pub mod models;
pub mod sqlite;
pub mod storage;
pub mod vector_search;

pub use config::Config;
pub use error::{FoundationError, Result};
pub use fs_store::FileStore;
pub use models::*;
pub use sqlite::{KnowledgeResult, SqliteDb};
pub use storage::{HealthStatus, Storage};
pub use vector_search::{
    cosine_similarity,
    Embedding,
    VectorDocument,
    VectorSearchResult,
    SimpleVectorIndex,
    VectorSearchError,
};
