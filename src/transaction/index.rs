use idb::Index;
use wasm_bindgen::JsValue;

use crate::{Direction, KeyRange, Result};

/// Index of an object store.
pub struct StoreIndex {
    pub(crate) index: Index,
}

impl StoreIndex {
    /// Returns name of the index
    pub fn name(&self) -> String {
        self.index.name()
    }

    /// Returns weather the index has unique enabled
    pub fn unique(&self) -> bool {
        self.index.unique()
    }

    /// Returns weather the index has multi entry enabled
    pub fn multi_entry(&self) -> bool {
        self.index.multi_entry()
    }

    /// Gets a value from the store with given key
    pub async fn get(&self, key: JsValue) -> Result<Option<JsValue>> {
        self.index.get(key)?.await.map_err(Into::into)
    }

    /// Retrieves the keys of all objects inside the index
    /// See: [MDN:IDBIndex/getAllKeys](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getAllKeys)
    pub async fn get_all_keys(
        &self,
        key_range: Option<KeyRange>,
        limit: Option<u32>,
    ) -> Result<Vec<JsValue>> {
        self.index
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
        self.index
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
        let cursor = self
            .index
            .open_cursor(key_range.map(Into::into), direction)?
            .await?;

        match cursor {
            None => Ok(Vec::new()),
            Some(cursor) => {
                let mut cursor = cursor.into_managed();

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
                                (Some(key), Some(value)) => {
                                    result.push((key, value));
                                    cursor.next(None).await?;
                                }
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
                                (Some(key), Some(value)) => {
                                    result.push((key, value));
                                    cursor.next(None).await?;
                                }
                                _ => break,
                            }
                        }
                    }
                }

                Ok(result)
            }
        }
    }

    /// Counts the number of key value pairs in the store
    pub async fn count(&self, key_range: Option<KeyRange>) -> Result<u32> {
        self.index
            .count(key_range.map(Into::into))?
            .await
            .map_err(Into::into)
    }
}
