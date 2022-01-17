use wasm_bindgen::JsValue;
use web_sys::IdbIndex;

use crate::{
    request::{wait_cursor_request, wait_request},
    Direction, Error, KeyRange, Result,
};

/// Index of an object store.
pub struct StoreIndex {
    pub(crate) idb_index: IdbIndex,
}

impl StoreIndex {
    /// Creates a new instance of `StoreIndex`.
    pub(crate) fn new(idb_index: IdbIndex) -> Self {
        Self { idb_index }
    }

    /// Returns name of the index
    pub fn name(&self) -> String {
        self.idb_index.name()
    }

    /// Returns weather the index has unique enabled
    pub fn unique(&self) -> bool {
        self.idb_index.unique()
    }

    /// Returns weather the index has multi entry enabled
    pub fn multi_entry(&self) -> bool {
        self.idb_index.multi_entry()
    }

    /// Gets a value from the store with given key
    pub async fn get(&self, key: &JsValue) -> Result<JsValue> {
        let request = self
            .idb_index
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
                .idb_index
                .open_cursor_with_range_and_direction(key_range.as_ref(), direction.into()),
            (Some(key_range), None) => self.idb_index.open_cursor_with_range(key_range.as_ref()),
            (None, Some(direction)) => self
                .idb_index
                .open_cursor_with_range_and_direction(&JsValue::null(), direction.into()),
            _ => self.idb_index.open_cursor(),
        }
        .map_err(Error::IndexedDbRequestError)?;

        wait_cursor_request(request, limit, offset, Error::IndexedDbRequestError).await
    }

    /// Counts the number of key value pairs in the store
    pub async fn count(&self, key_range: Option<&KeyRange>) -> Result<u32> {
        let request = match key_range {
            Some(key_range) => self.idb_index.count_with_key(key_range.as_ref()),
            None => self.idb_index.count(),
        }
        .map_err(Error::IndexedDbRequestError)?;

        let result = wait_request(request, Error::IndexedDbRequestError).await?;

        result
            .as_f64()
            .and_then(num_traits::cast)
            .ok_or(Error::UnexpectedJsType)
    }
}
