use std::collections::HashSet;

use js_sys::Array;
use wasm_bindgen::prelude::*;
use web_sys::{IdbDatabase, IdbObjectStoreParameters, IdbOpenDbRequest};

use crate::{Error, Index, Result};

/// An object store builder.
pub struct ObjectStore {
    pub(crate) name: String,
    pub(crate) key_path: Vec<String>,
    pub(crate) auto_increment: Option<bool>,
    pub(crate) indexes: Vec<Index>,
}

impl ObjectStore {
    /// Creates a new object store with given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            key_path: Vec::new(),
            auto_increment: None,
            indexes: Vec::new(),
        }
    }

    /// Specify key path for the object store
    pub fn key_path(mut self, key_path: &str) -> Self {
        self.key_path = vec![key_path.to_owned()];
        self
    }

    /// Specify key path array for the object store
    pub fn key_path_array<'a>(mut self, key_path_array: impl IntoIterator<Item = &'a str>) -> Self {
        self.key_path = key_path_array.into_iter().map(ToOwned::to_owned).collect();
        self
    }

    /// Specify whether the object store should auto increment keys
    pub fn auto_increment(mut self, auto_increment: bool) -> Self {
        self.auto_increment = Some(auto_increment);
        self
    }

    /// Add an index to the object store
    pub fn add_index(mut self, index: Index) -> Self {
        self.indexes.push(index);
        self
    }
}

impl ObjectStore {
    pub(crate) fn create(
        self,
        idb_open_request: &IdbOpenDbRequest,
        idb: &IdbDatabase,
    ) -> Result<()> {
        let mut index_names = self.index_names();

        let object_store = if idb.object_store_names().contains(&self.name) {
            let transaction = idb_open_request
                .transaction()
                .ok_or(Error::TransactionNotFound)?;

            transaction
                .object_store(&self.name)
                .map_err(Error::ObjectStoreOpenFailed)?
        } else {
            let mut params = IdbObjectStoreParameters::new();

            if let Some(auto_increment) = self.auto_increment {
                params.auto_increment(auto_increment);
            }

            match self.key_path.len() {
                0 => {
                    params.key_path(None);
                }
                1 => {
                    params.key_path(Some(&(&self.key_path[0]).into()));
                }
                _ => {
                    params.key_path(Some(
                        &self
                            .key_path
                            .into_iter()
                            .map(|key| JsValue::from_str(&key))
                            .collect::<Array>()
                            .into(),
                    ));
                }
            }

            idb.create_object_store_with_optional_parameters(&self.name, &params)
                .map_err(Error::ObjectStoreCreationFailed)?
        };

        for index in self.indexes {
            index.create(&object_store)?;
        }

        let db_index_names = object_store.index_names();
        let mut indexes_to_remove = Vec::new();

        for index in 0..db_index_names.length() {
            let db_index_name = db_index_names.get(index).ok_or_else(|| {
                Error::ObjectStoreCreationFailed("unable to get index name".into())
            })?;

            if index_names.contains(&db_index_name) {
                index_names.remove(&db_index_name);
            } else {
                indexes_to_remove.push(db_index_name);
            }
        }

        for index_name in indexes_to_remove {
            object_store
                .delete_index(&index_name)
                .map_err(Error::ObjectStoreCreationFailed)?;
        }

        Ok(())
    }

    pub(crate) fn index_names(&self) -> HashSet<String> {
        self.indexes
            .iter()
            .map(|index| index.name.clone())
            .collect()
    }
}
