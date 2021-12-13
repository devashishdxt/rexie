#[cfg(feature = "js")]
use std::rc::Rc;

#[cfg(not(feature = "js"))]
use js_sys::Array;
#[cfg(feature = "js")]
use js_sys::Promise;
#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::IdbObjectStore;

use crate::{utils::wait_request, ErrorType, KeyRange, Result, StoreIndex};

#[cfg_attr(feature = "js", wasm_bindgen)]
#[cfg_attr(feature = "js", derive(Clone))]
pub struct Store {
    #[cfg(not(feature = "js"))]
    pub(crate) idb_store: IdbObjectStore,

    #[cfg(feature = "js")]
    pub(crate) idb_store: Rc<IdbObjectStore>,
}

impl Store {
    pub fn new(idb_store: IdbObjectStore) -> Self {
        #[cfg(feature = "js")]
        let idb_store = Rc::new(idb_store);

        Self { idb_store }
    }
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl Store {
    pub fn index(&self, name: &str) -> Result<StoreIndex> {
        let idb_index = self
            .idb_store
            .index(name)
            .map_err(|js_value| ErrorType::IndexOpenFailed.into_error().set_inner(js_value))?;

        Ok(StoreIndex::new(idb_index))
    }
}

#[cfg(not(feature = "js"))]
impl Store {
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

    pub async fn put(&self, key: impl Into<JsValue>, value: impl Into<JsValue>) -> Result<()> {
        self.put_js(&key.into(), &value.into()).await
    }

    pub async fn put_value(&self, value: impl Into<JsValue>) -> Result<()> {
        self.put_value_js(&value.into()).await
    }

    pub async fn add(&self, key: impl Into<JsValue>, value: impl Into<JsValue>) -> Result<()> {
        self.add_js(&key.into(), &value.into()).await
    }

    pub async fn add_value(&self, value: impl Into<JsValue>) -> Result<()> {
        self.add_value_js(&value.into()).await
    }

    pub async fn delete(&self, key: impl Into<JsValue>) -> Result<()> {
        self.delete_js(&key.into()).await
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

    pub async fn clear(&self) -> Result<()> {
        self.clear_js().await
    }
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl Store {
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

    pub fn put(&self, key: JsValue, value: JsValue) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            this.put_js(&key, &value).await.map(|()| JsValue::UNDEFINED)
        })
    }

    #[wasm_bindgen(js_name = "putValue")]
    pub fn put_value(&self, value: JsValue) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            this.put_value_js(&value).await.map(|()| JsValue::UNDEFINED)
        })
    }

    pub fn add(&self, key: JsValue, value: JsValue) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            this.add_js(&key, &value).await.map(|()| JsValue::UNDEFINED)
        })
    }

    #[wasm_bindgen(js_name = "addValue")]
    pub fn add_value(&self, value: JsValue) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            this.add_value_js(&value).await.map(|()| JsValue::UNDEFINED)
        })
    }

    pub fn delete(&self, key: JsValue) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            this.delete_js(&key).await.map(|()| JsValue::UNDEFINED)
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

    pub fn clear(&self) -> Promise {
        let this = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            this.clear_js().await.map(|()| JsValue::UNDEFINED)
        })
    }
}

impl Store {
    async fn get_js(&self, key: &JsValue) -> Result<JsValue> {
        let request = self
            .idb_store
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
            .idb_store
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
            None => self.idb_store.get_all_with_key(key_range.as_ref()),
            Some(limit) => self
                .idb_store
                .get_all_with_key_and_limit(key_range.as_ref(), limit),
        }
        .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await?;

        let value = request
            .result()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        Ok(value)
    }

    async fn put_js(&self, key: &JsValue, value: &JsValue) -> Result<()> {
        let request = self
            .idb_store
            .put_with_key(value, key)
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await
    }

    async fn put_value_js(&self, value: &JsValue) -> Result<()> {
        let request = self
            .idb_store
            .put(value)
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await
    }

    async fn add_js(&self, key: &JsValue, value: &JsValue) -> Result<()> {
        let request = self
            .idb_store
            .add_with_key(value, key)
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await
    }

    async fn add_value_js(&self, value: &JsValue) -> Result<()> {
        let request = self
            .idb_store
            .add(value)
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await
    }

    async fn delete_js(&self, key: &JsValue) -> Result<()> {
        let request = self
            .idb_store
            .delete(key)
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await
    }

    async fn count_js(&self) -> Result<JsValue> {
        let request = self
            .idb_store
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
            .idb_store
            .count_with_key(key_range.as_ref())
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await?;

        let count = request
            .result()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        Ok(count)
    }

    async fn clear_js(&self) -> Result<()> {
        let request = self
            .idb_store
            .clear()
            .map_err(|js_value| ErrorType::IndexedDBError.into_error().set_inner(js_value))?;

        wait_request(&request).await
    }
}
