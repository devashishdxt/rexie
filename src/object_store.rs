use idb::builder::ObjectStoreBuilder;

use crate::{Index, KeyPath};

/// An object store builder.
pub struct ObjectStore {
    pub(crate) builder: ObjectStoreBuilder,
}

impl ObjectStore {
    /// Creates a new object store with given name
    pub fn new(name: &str) -> Self {
        Self {
            builder: ObjectStoreBuilder::new(name),
        }
    }

    /// Specify key path for the object store
    pub fn key_path(mut self, key_path: &str) -> Self {
        self.builder = self.builder.key_path(Some(KeyPath::new_single(key_path)));
        self
    }

    /// Specify key path array for the object store
    pub fn key_path_array<'a>(mut self, key_path_array: impl IntoIterator<Item = &'a str>) -> Self {
        self.builder = self
            .builder
            .key_path(Some(KeyPath::new_array(key_path_array)));
        self
    }

    /// Specify whether the object store should auto increment keys
    pub fn auto_increment(mut self, auto_increment: bool) -> Self {
        self.builder = self.builder.auto_increment(auto_increment);
        self
    }

    /// Add an index to the object store
    pub fn add_index(mut self, index: Index) -> Self {
        self.builder = self.builder.add_index(index.builder);
        self
    }
}
