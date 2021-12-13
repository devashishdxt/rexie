#[cfg(feature = "js")]
use std::rc::Rc;

#[cfg(not(feature = "js"))]
use js_sys::Array;
#[cfg(feature = "js")]
use js_sys::Promise;
#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::IdbIndex;

use crate::{utils::wait_request, ErrorType, KeyRange, Result};

#[cfg_attr(feature = "js", wasm_bindgen)]
#[cfg_attr(feature = "js", derive(Clone))]
pub struct StoreIndex {
    #[cfg(not(feature = "js"))]
    pub(crate) idb_index: IdbIndex,

    #[cfg(feature = "js")]
    pub(crate) idb_index: Rc<IdbIndex>,
}

impl StoreIndex {
    pub fn new(idb_index: IdbIndex) -> Self {
        #[cfg(feature = "js")]
        let idb_index = Rc::new(idb_index);

        Self { idb_index }
    }
}

#[cfg(not(feature = "js"))]
impl StoreIndex {
    pub async fn get(&self, key: impl Into<JsValue>) -> Result<JsValue> {
        self.get_js(&key.into()).await
    }

    pub async fn get_all(&self) -> Result<Vec<JsValue>> {
        self.get_all_js()
            .await
            .map(|array| Array::from(&array).to_vec())
    }

    pub async fn get_all_key_range(
        &self,
        key_range: KeyRange,
        limit: Option<u32>,
    ) -> Result<Vec<JsValue>> {
        self.get_all_key_range_js(&key_range, limit)
            .await
            .map(|array| Array::from(&array).to_vec())
    }

    pub async fn count(&self) -> Result<u32> {
        let result = self.count_js().await?;

        let count = result
            .as_f64()
            .map(|count| count as u32)
            .ok_or(ErrorType::IndexedDBError)?;

        Ok(count)
    }

    pub async fn count_key_range(&self, key_range: &KeyRange) -> Result<u32> {
        let result = self.count_key_range_js(key_range).await?;

        let count = result
            .as_f64()
            .map(|count| count as u32)
            .ok_or(ErrorType::IndexedDBError)?;

        Ok(count)
    }
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl StoreIndex {
    pub fn get(&self, key: JsValue) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move { this.get_js(&key).await })
    }

    #[wasm_bindgen(js_name = "getAll")]
    pub fn get_all(&self) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move { this.get_all_js().await })
    }

    #[wasm_bindgen(js_name = "getAllKeyRange")]
    pub fn get_all_key_range(&self, key_range: KeyRange, limit: Option<u32>) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            this.get_all_key_range_js(&key_range, limit).await
        })
    }

    pub fn count(&self) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move { this.count_js().await })
    }

    #[wasm_bindgen(js_name = "countKeyRange")]
    pub fn count_key_range(&self, key_range: KeyRange) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(
            async move { this.count_key_range_js(&key_range).await },
        )
    }
}

impl StoreIndex {
    async fn get_js(&self, key: &JsValue) -> Result<JsValue> {
        let request = self
            .idb_index
            .get(key)
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await?;

        let value = request
            .result()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        Ok(value)
    }

    async fn get_all_js(&self) -> Result<JsValue> {
        let request = self
            .idb_index
            .get_all()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await?;

        let value = request
            .result()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        Ok(value)
    }

    async fn get_all_key_range_js(
        &self,
        key_range: &KeyRange,
        limit: Option<u32>,
    ) -> Result<JsValue> {
        let request = match limit {
            None => self.idb_index.get_all_with_key(key_range.as_ref()),
            Some(limit) => self
                .idb_index
                .get_all_with_key_and_limit(key_range.as_ref(), limit),
        }
        .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await?;

        let value = request
            .result()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        Ok(value)
    }

    async fn count_js(&self) -> Result<JsValue> {
        let request = self
            .idb_index
            .count()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await?;

        let count = request
            .result()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        Ok(count)
    }

    async fn count_key_range_js(&self, key_range: &KeyRange) -> Result<JsValue> {
        let request = self
            .idb_index
            .count_with_key(key_range.as_ref())
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await?;

        let count = request
            .result()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        Ok(count)
    }
}
