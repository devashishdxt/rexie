mod index;
mod store;

pub use self::{index::StoreIndex, store::Store};

#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::{throw_str, JsCast, JsValue, UnwrapThrowExt};
#[cfg(feature = "js")]
use web_sys::DomStringList;
use web_sys::IdbTransaction;

use crate::{
    observer::{Observer, ResultObserver},
    ErrorType, Result,
};

#[cfg(not(feature = "js"))]
#[derive(Debug, Clone, Copy)]
pub enum TransactionMode {
    ReadOnly,
    ReadWrite,
    ReadWriteFlush,
    Cleanup,
    VersionChange,
}

#[cfg(feature = "js")]
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum TransactionMode {
    ReadOnly = "readonly",
    ReadWrite = "readwrite",
    ReadWriteFlush = "readwriteflush",
    Cleanup = "cleanup",
    VersionChange = "versionchange",
}

impl From<TransactionMode> for web_sys::IdbTransactionMode {
    fn from(mode: TransactionMode) -> Self {
        match mode {
            TransactionMode::ReadOnly => Self::Readonly,
            TransactionMode::ReadWrite => Self::Readwrite,
            TransactionMode::ReadWriteFlush => Self::Readwriteflush,
            TransactionMode::Cleanup => Self::Cleanup,
            TransactionMode::VersionChange => Self::Versionchange,
            #[cfg(feature = "js")]
            _ => throw_str("invalid transaction mode"),
        }
    }
}

impl From<web_sys::IdbTransactionMode> for TransactionMode {
    fn from(mode: web_sys::IdbTransactionMode) -> Self {
        match mode {
            web_sys::IdbTransactionMode::Readonly => Self::ReadOnly,
            web_sys::IdbTransactionMode::Readwrite => Self::ReadWrite,
            web_sys::IdbTransactionMode::Readwriteflush => Self::ReadWriteFlush,
            web_sys::IdbTransactionMode::Cleanup => Self::Cleanup,
            web_sys::IdbTransactionMode::Versionchange => Self::VersionChange,
            _ => throw_str("invalid transaction mode"),
        }
    }
}

#[cfg_attr(feature = "js", wasm_bindgen)]
pub struct Transaction {
    pub(crate) idb_transaction: IdbTransaction,
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl Transaction {
    #[cfg_attr(feature = "js", wasm_bindgen(getter))]
    pub fn mode(&self) -> TransactionMode {
        self.idb_transaction.mode().unwrap_throw().into()
    }

    #[cfg(feature = "js")]
    #[wasm_bindgen(getter, js_name = "storeNames")]
    pub fn store_names(&self) -> DomStringList {
        self.idb_transaction.object_store_names()
    }

    #[cfg(not(feature = "js"))]
    pub fn store_names(&self) -> Vec<String> {
        let list = self.idb_transaction.object_store_names();

        let mut result = Vec::new();

        for index in 0..list.length() {
            if let Some(s) = list.get(index) {
                result.push(s);
            }
        }

        result
    }

    pub async fn abort(self) -> Result<()> {
        let abort_observer = Observer::new(());
        self.idb_transaction
            .set_onabort(Some(abort_observer.get_callback()));

        self.idb_transaction.abort().map_err(|js_value| {
            ErrorType::TransactionExecutionFailed
                .into_error()
                .set_inner(js_value)
        })?;

        abort_observer.finish().await
    }

    pub async fn finish(self) -> Result<()> {
        let observer = ResultObserver::new((), |event| {
            let target = event.target().unwrap_throw();
            let request: &IdbTransaction = AsRef::<JsValue>::as_ref(&target).unchecked_ref();

            #[allow(clippy::useless_conversion)]
            match request.error() {
                Some(exception) => Err(ErrorType::TransactionExecutionFailed
                    .into_error()
                    .set_inner(exception.into())
                    .into()),
                None => Err(ErrorType::IndexedDBError.into()),
            }
        });

        self.idb_transaction
            .set_oncomplete(Some(observer.get_success_callback()));
        self.idb_transaction
            .set_onerror(Some(observer.get_error_callback()));

        observer.finish().await
    }

    pub fn store(&self, store_name: &str) -> Result<Store> {
        Ok(Store::new(
            self.idb_transaction
                .object_store(store_name)
                .map_err(|js_value| {
                    ErrorType::ObjectStoreOpenFailed
                        .into_error()
                        .set_inner(js_value)
                })?,
        ))
    }
}
