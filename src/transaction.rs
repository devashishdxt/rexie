mod index;
mod store;

pub use self::{index::StoreIndex, store::Store};

#[cfg(feature = "js")]
use wasm_bindgen::{prelude::*, throw_str};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
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

#[cfg_attr(feature = "js", wasm_bindgen)]
pub struct Transaction {
    pub(crate) idb_transaction: IdbTransaction,
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl Transaction {
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
