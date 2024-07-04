use idb::ObjectStore;
use wasm_bindgen::JsValue;

use crate::{Direction, Error, KeyPath, KeyRange, Result, StoreIndex};

/// An object store.
pub struct Store {
    pub(crate) object_store: ObjectStore,
}

impl Store {
    /// Returns whether the store has auto increment enabled
    /// MDN Reference: [IDBObjectStore.autoIncrement](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/autoIncrement)
    pub fn auto_increment(&self) -> bool {
        self.object_store.auto_increment()
    }

    /// Returns the name of the store
    /// MDN Reference: [IDBObjectStore.name](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/name)
    pub fn name(&self) -> String {
        self.object_store.name()
    }

    /// Returns the key path of the store
    /// MDN Reference: [IDBObjectStore.keyPath](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/keyPath)
    pub fn key_path(&self) -> Result<Option<KeyPath>> {
        self.object_store.key_path().map_err(Into::into)
    }

    /// Returns all the index names of the store
    /// MDN Reference: [IDBObjectStore/indexNames](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/indexNames)
    pub fn index_names(&self) -> Vec<String> {
        self.object_store.index_names()
    }

    /// Returns index of the store with given name
    /// MDN Reference: [IDBObjectStore/index](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/index)
    pub fn index(&self, name: &str) -> Result<StoreIndex> {
        let index = self.object_store.index(name)?;
        Ok(StoreIndex { index })
    }

    /// Gets a value from the store with given key
    /// MDN Reference: [IDBObjectStore/get](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/get)
    pub async fn get(&self, key: JsValue) -> Result<Option<JsValue>> {
        self.object_store.get(key)?.await.map_err(Into::into)
    }

    /// Checks if a given key exists within the store
    /// MDN Reference: [IDBObjectStore/getKey](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/getKey)
    pub async fn key_exists(&self, key: JsValue) -> Result<bool> {
        self.object_store
            .get_key(key)?
            .await
            .map(|key| key.is_some())
            .map_err(Into::into)
    }

    /// Retrieves record keys for all objects in the object store matching the specified
    /// parameter or all objects in the store if no parameters are given.
    /// MDN Reference: [IDBStore/getAllKeys](https://developer.mozilla.org/en-US/docs/Web/API/IDBStore/getAllKeys)
    pub async fn get_all_keys(
        &self,
        key_range: Option<KeyRange>,
        limit: Option<u32>,
    ) -> Result<Vec<JsValue>> {
        self.object_store
            .get_all_keys(key_range.map(Into::into), limit)?
            .await
            .map_err(Into::into)
    }

    /// Gets all values from the store with given key range and limit
    pub async fn get_all(
        &self,
        key_range: Option<KeyRange>,
        limit: Option<u32>,
    ) -> Result<Vec<JsValue>> {
        self.object_store
            .get_all(key_range.map(Into::into), limit)?
            .await
            .map_err(Into::into)
    }

    /// Scans all key-value pairs from the store with given key range, limit, offset and direction
    pub async fn scan(
        &self,
        key_range: Option<KeyRange>,
        limit: Option<u32>,
        offset: Option<u32>,
        direction: Option<Direction>,
    ) -> Result<Vec<(JsValue, JsValue)>> {
        let mut cursor = self
            .object_store
            .open_cursor(key_range.map(Into::into), direction)?
            .await?
            .ok_or(Error::CursorNotFound)?
            .into_managed();

        let mut result = Vec::new();

        match limit {
            Some(limit) => {
                if let Some(offset) = offset {
                    cursor.advance(offset).await?;
                }

                for _ in 0..limit {
                    let key = cursor.key()?;
                    let value = cursor.value()?;

                    match (key, value) {
                        (Some(key), Some(value)) => result.push((key, value)),
                        _ => break,
                    }
                }
            }
            None => {
                if let Some(offset) = offset {
                    cursor.advance(offset).await?;
                }

                loop {
                    let key = cursor.key()?;
                    let value = cursor.value()?;

                    match (key, value) {
                        (Some(key), Some(value)) => result.push((key, value)),
                        _ => break,
                    }
                }
            }
        }

        Ok(result)
    }

    /// Adds a key value pair in the store. Note that the key can be `None` if store has auto increment enabled.
    pub async fn add(&self, value: &JsValue, key: Option<&JsValue>) -> Result<JsValue> {
        self.object_store.add(value, key)?.await.map_err(Into::into)
    }

    /// Puts (adds or updates) a key value pair in the store.
    pub async fn put(&self, value: &JsValue, key: Option<&JsValue>) -> Result<JsValue> {
        self.object_store.put(value, key)?.await.map_err(Into::into)
    }

    /// Deletes a key value pair from the store
    pub async fn delete(&self, key: JsValue) -> Result<()> {
        self.object_store.delete(key)?.await.map_err(Into::into)
    }

    /// Counts the number of key value pairs in the store
    pub async fn count(&self, key_range: Option<KeyRange>) -> Result<u32> {
        self.object_store
            .count(key_range.map(Into::into))?
            .await
            .map_err(Into::into)
    }

    /// Deletes all key value pairs from the store
    pub async fn clear(&self) -> Result<()> {
        self.object_store.clear()?.await.map_err(Into::into)
    }
}
