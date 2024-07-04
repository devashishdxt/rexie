use idb::builder::IndexBuilder;

use crate::KeyPath;

/// An index builder.
pub struct Index {
    pub(crate) builder: IndexBuilder,
}

impl Index {
    /// Creates a new index with given name and key path
    pub fn new(name: &str, key_path: &str) -> Self {
        Self {
            builder: IndexBuilder::new(name.to_owned(), KeyPath::new_single(key_path)),
        }
    }

    /// Creates a new index with given name and key path array
    pub fn new_array<'a>(name: &str, key_path_array: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            builder: IndexBuilder::new(name.to_owned(), KeyPath::new_array(key_path_array)),
        }
    }

    /// Specify whether the index should be unique
    pub fn unique(mut self, unique: bool) -> Self {
        self.builder = self.builder.unique(unique);
        self
    }

    /// Specify whether the index should be multi-entry, i.e., type of the value contained in key path is an array
    pub fn multi_entry(mut self, multi_entry: bool) -> Self {
        self.builder = self.builder.multi_entry(multi_entry);
        self
    }
}
