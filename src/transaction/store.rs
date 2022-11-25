use wasm_bindgen::prelude::*;
use web_sys::IdbObjectStore;

use crate::{
    request::{wait_cursor_request, wait_request},
    Direction, Error, KeyRange, Result, StoreIndex,
};

/// An object store.
pub struct Store {
    pub(crate) idb_store: IdbObjectStore,
}

impl Store {
    /// Creates a new instance of `Store`.
    pub(crate) fn new(idb_store: IdbObjectStore) -> Self {
        Self { idb_store }
    }

    /// Returns weather the store has auto increment enabled
    pub fn auto_increment(&self) -> bool {
        self.idb_store.auto_increment()
    }

    /// Returns the name of the store
    pub fn name(&self) -> String {
        self.idb_store.name()
    }

    /// Returns the key path of the store
    pub fn key_path(&self) -> Result<Option<String>> {
        self.idb_store
            .key_path()
            .map(|js_value| js_value.as_string())
            .map_err(Error::IndexedDbRequestError)
    }

    /// Returns all the index names of the store
    pub fn index_names(&self) -> Vec<String> {
        let list = self.idb_store.index_names();

        let mut result = Vec::new();

        for index in 0..list.length() {
            if let Some(s) = list.get(index) {
                result.push(s);
            }
        }

        result
    }

    /// Returns index of the store with given name
    pub fn index(&self, name: &str) -> Result<StoreIndex> {
        let idb_index = self.idb_store.index(name).map_err(Error::IndexOpenFailed)?;

        Ok(StoreIndex::new(idb_index))
    }

    /// Gets a value from the store with given key
    pub async fn get(&self, key: &JsValue) -> Result<JsValue> {
        let request = self
            .idb_store
            .get(key)
            .map_err(Error::IndexedDbRequestError)?;

        wait_request(request, Error::IndexedDbRequestError).await
    }

    /// Gets all key-value pairs from the store with given key range, limit, offset and direction
    pub async fn get_all(
        &self,
        key_range: Option<&KeyRange>,
        limit: Option<u32>,
        offset: Option<u32>,
        direction: Option<Direction>,
    ) -> Result<Vec<(JsValue, JsValue)>> {
        let request = match (key_range, direction) {
            (Some(key_range), Some(direction)) => self
                .idb_store
                .open_cursor_with_range_and_direction(key_range.as_ref(), direction.into()),
            (Some(key_range), None) => self.idb_store.open_cursor_with_range(key_range.as_ref()),
            (None, Some(direction)) => self
                .idb_store
                .open_cursor_with_range_and_direction(&JsValue::null(), direction.into()),
            _ => self.idb_store.open_cursor(),
        }
        .map_err(Error::IndexedDbRequestError)?;

        wait_cursor_request(request, limit, offset, Error::IndexedDbRequestError).await
    }

    /// Retrieves record keys for all objects in the object store.
    pub async fn get_all_keys(
        &self,
        key_range: Option<&KeyRange>,
        limit: Option<u32>,
    ) -> Result<JsValue> {
        let request = match (key_range, limit) {
            (Some(key_range), None) => self.idb_store.get_all_keys_with_key(key_range.as_ref()),
            (Some(key_range), Some(limit)) => self
                .idb_store
                .get_all_keys_with_key_and_limit(key_range.as_ref(), limit),
            _ => self.idb_store.get_all_keys(),
        }
        .map_err(Error::IndexedDbRequestError)?;
        wait_request(request, Error::IndexedDbRequestError).await
    }

    /// Adds a key value pair in the store. Note that the key can be `None` if store has auto increment enabled.
    pub async fn add(&self, value: &JsValue, key: Option<&JsValue>) -> Result<JsValue> {
        let request = match key {
            Some(key) => self.idb_store.add_with_key(value, key),
            None => self.idb_store.add(value),
        }
        .map_err(Error::IndexedDbRequestError)?;

        wait_request(request, Error::IndexedDbRequestError).await
    }

    /// Puts (adds or updates) a key value pair in the store.
    pub async fn put(&self, value: &JsValue, key: Option<&JsValue>) -> Result<JsValue> {
        let request = match key {
            Some(key) => self.idb_store.put_with_key(value, key),
            None => self.idb_store.put(value),
        }
        .map_err(Error::IndexedDbRequestError)?;

        wait_request(request, Error::IndexedDbRequestError).await
    }

    /// Deletes a key value pair from the store
    pub async fn delete(&self, key: &JsValue) -> Result<()> {
        let request = self
            .idb_store
            .delete(key)
            .map_err(Error::IndexedDbRequestError)?;

        wait_request(request, Error::IndexedDbRequestError).await?;

        Ok(())
    }

    /// Counts the number of key value pairs in the store
    pub async fn count(&self, key_range: Option<&KeyRange>) -> Result<u32> {
        let request = match key_range {
            Some(key_range) => self.idb_store.count_with_key(key_range.as_ref()),
            None => self.idb_store.count(),
        }
        .map_err(Error::IndexedDbRequestError)?;

        let result = wait_request(request, Error::IndexedDbRequestError).await?;

        result
            .as_f64()
            .and_then(num_traits::cast)
            .ok_or(Error::UnexpectedJsType)
    }

    /// Deletes all key value pairs from the store
    pub async fn clear(&self) -> Result<()> {
        let request = self
            .idb_store
            .clear()
            .map_err(Error::IndexedDbRequestError)?;

        wait_request(request, Error::IndexedDbRequestError)
            .await
            .map(|_| ())
    }
}
