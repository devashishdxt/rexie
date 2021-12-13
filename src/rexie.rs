use js_sys::Array;
#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "js")]
use web_sys::DomStringList;
use web_sys::IdbDatabase;

use crate::{ErrorType, Result, RexieBuilder, Transaction, TransactionMode};

#[cfg(not(feature = "js"))]
type StoreNames = Vec<String>;

#[cfg(feature = "js")]
type StoreNames = Array;

#[cfg_attr(feature = "js", wasm_bindgen)]
pub struct Rexie {
    pub(crate) db: IdbDatabase,
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl Rexie {
    pub fn builder(name: &str) -> RexieBuilder {
        RexieBuilder::new(name)
    }

    #[cfg_attr(feature = "js", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.db.name()
    }

    #[cfg_attr(feature = "js", wasm_bindgen(getter))]
    pub fn version(&self) -> f64 {
        self.db.version()
    }

    #[cfg(feature = "js")]
    #[wasm_bindgen(getter, js_name = "storeNames")]
    pub fn store_names(&self) -> DomStringList {
        self.db.object_store_names()
    }

    #[cfg(not(feature = "js"))]
    pub fn store_names(&self) -> Vec<String> {
        let list = self.db.object_store_names();

        let mut result = Vec::new();

        for index in 0..list.length() {
            if let Some(s) = list.get(index) {
                result.push(s);
            }
        }

        result
    }

    pub fn transaction(
        &self,
        store_names: StoreNames,
        mode: TransactionMode,
    ) -> Result<Transaction> {
        #[cfg(not(feature = "js"))]
        let store_names: Array = store_names
            .into_iter()
            .map(wasm_bindgen::JsValue::from)
            .collect();

        let idb_transaction = self
            .db
            .transaction_with_str_sequence_and_mode(&store_names, mode.into())
            .map_err(|js_value| {
                ErrorType::TransactionOpenFailed
                    .into_error()
                    .set_inner(js_value)
            })?;

        Ok(Transaction { idb_transaction })
    }

    pub fn close(self) {
        self.db.close();
    }
}
