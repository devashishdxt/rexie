use idb::{builder::DatabaseBuilder, Factory};

use crate::{ObjectStore, Result, Rexie};

/// Builder for creating a new database.
pub struct RexieBuilder {
    name: String,
    builder: DatabaseBuilder,
}

impl RexieBuilder {
    /// Creates a new database builder with given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            builder: DatabaseBuilder::new(name),
        }
    }

    /// Specify version of the database.
    pub fn version(mut self, version: u32) -> Self {
        self.builder = self.builder.version(version);
        self
    }

    /// Add an object store to the database.
    pub fn add_object_store(mut self, object_store: ObjectStore) -> Self {
        self.builder = self.builder.add_object_store(object_store.builder);
        self
    }

    /// Build the database.
    pub async fn build(self) -> Result<Rexie> {
        let database = self.builder.build().await?;
        Ok(Rexie { database })
    }

    /// Delete the database.
    pub async fn delete(self) -> Result<()> {
        let factory = Factory::new()?;
        factory.delete(&self.name)?.await.map_err(Into::into)
    }
}
