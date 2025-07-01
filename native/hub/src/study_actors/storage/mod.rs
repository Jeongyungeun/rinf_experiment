mod sled_storage;
pub use sled_storage::SledStorage;

use async_trait::async_trait;
use crate::study_actors::messages::StorageError;

#[async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn save(&self, key: &str, data: &[u8]) -> Result<(), StorageError>;
    async fn load(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    async fn exists(&self, key: &str) -> Result<bool, StorageError>;
}
