//! Repository trait and common functionality

use crate::Result;

/// Generic repository trait for CRUD operations
pub trait Repository<T, ID> {
    /// Find an entity by its ID
    fn find_by_id(&self, id: ID) -> Result<Option<T>>;

    /// Find all entities
    fn find_all(&self) -> Result<Vec<T>>;

    /// Create a new entity
    fn create(&self, entity: &T) -> Result<T>;

    /// Update an existing entity
    fn update(&self, entity: &T) -> Result<T>;

    /// Delete an entity by its ID
    fn delete(&self, id: ID) -> Result<()>;
}
