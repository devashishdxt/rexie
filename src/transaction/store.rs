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

    /// Returns whether the store has auto increment enabled
    /// MDN Reference: [IDBObjectStore.autoIncrement](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/autoIncrement)
    pub fn auto_increment(&self) -> bool {
        self.idb_store.auto_increment()
    }

    /// Returns the name of the store
    /// MDN Reference: [IDBObjectStore.name](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/name)
    pub fn name(&self) -> String {
        self.idb_store.name()
    }

    /// Returns the key path of the store
    /// MDN Reference: [IDBObjectStore.keyPath](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/keyPath)
    pub fn key_path(&self) -> Result<Option<String>> {
        self.idb_store
            .key_path()
            .map(|js_value| js_value.as_string())
            .map_err(Error::IndexedDbRequestError)
    }

    /// Returns all the index names of the store
    /// MDN Reference: [IDBObjectStore/indexNames](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/indexNames)
    pub fn index_names(&self) -> Vec<String> {
        let list = self.idb_store.index_names();
        let list_len = list.length();
        let mut result = Vec::with_capacity(list_len as usize);

        for index in 0..list_len {
            if let Some(s) = list.get(index) {
                result.push(s);
            }
        }

        result
    }

    /// Returns index of the store with given name
    /// MDN Reference: [IDBObjectStore/index](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/index)
    pub fn index(&self, name: &str) -> Result<StoreIndex> {
        let idb_index = self.idb_store.index(name).map_err(Error::IndexOpenFailed)?;

        Ok(StoreIndex::new(idb_index))
    }

    /// Gets a value from the store with given key
    /// MDN Reference: [IDBObjectStore/get](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/get)
    pub async fn get(&self, key: &JsValue) -> Result<Option<JsValue>> {
        let request = self
            .idb_store
            .get(key)
            .map_err(Error::IndexedDbRequestError)?;

        let response = wait_request(request, Error::IndexedDbRequestError).await?;
        if response.is_undefined() || response.is_null() {
            Ok(None)
        } else {
            Ok(Some(response))
        }
    }

    /// Checks if a given key exists within the store
    /// MDN Reference: [IDBObjectStore/getKey](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/getKey)
    pub async fn key_exists(&self, key: &JsValue) -> Result<bool> {
        let request = self
            .idb_store
            .get_key(key)
            .map_err(Error::IndexedDbRequestError)?;
        let result = wait_request(request, Error::IndexedDbRequestError).await?;
        Ok(result.as_bool().unwrap_or_default())
    }

    /// Retrieves record keys for all objects in the object store matching the specified
    /// parameter or all objects in the store if no parameters are given.
    /// MDN Reference: [IDBStore/getAllKeys](https://developer.mozilla.org/en-US/docs/Web/API/IDBStore/getAllKeys)
    pub async fn all_keys(
        &self,
        key_range: Option<&KeyRange>,
        limit: Option<u32>,
    ) -> Result<JsValue> {
        let request = match (key_range, limit) {
            (None, None) => self.idb_store.get_all_keys(),
            (None, Some(limit)) => self
                .idb_store
                .get_all_keys_with_key_and_limit(&JsValue::UNDEFINED, limit),
            (Some(key_range), None) => self.idb_store.get_all_keys_with_key(key_range.as_ref()),
            (Some(key_range), Some(limit)) => self
                .idb_store
                .get_all_keys_with_key_and_limit(key_range.as_ref(), limit),
        }
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
