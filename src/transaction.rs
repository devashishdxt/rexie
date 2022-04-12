mod index;
mod store;

pub use self::{index::StoreIndex, store::Store};

use wasm_bindgen::{prelude::*, throw_str};
use web_sys::IdbTransaction;

use crate::{
    request::{wait_request, wait_transaction_abort},
    Error, Result,
};

/// Different transaction modes for indexed db
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    ReadOnly,
    ReadWrite,
    ReadWriteFlush,
    Cleanup,
    VersionChange,
}

impl From<TransactionMode> for web_sys::IdbTransactionMode {
    fn from(mode: TransactionMode) -> Self {
        match mode {
            TransactionMode::ReadOnly => Self::Readonly,
            TransactionMode::ReadWrite => Self::Readwrite,
            TransactionMode::ReadWriteFlush => Self::Readwriteflush,
            TransactionMode::Cleanup => Self::Cleanup,
            TransactionMode::VersionChange => Self::Versionchange,
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

/// Transaction on the database
pub struct Transaction {
    pub(crate) idb_transaction: IdbTransaction,
}

impl Transaction {
    /// Returns mode of the transaction
    pub fn mode(&self) -> TransactionMode {
        self.idb_transaction
            .mode()
            .expect_throw("unable to get transaction mode")
            .into()
    }

    /// Returns names of all stores in the transaction
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

    /// Aborts a transaction
    pub async fn abort(self) -> Result<()> {
        wait_transaction_abort(self.idb_transaction).await
    }

    /// Commits a transaction
    ///
    /// # Note
    ///
    /// Note that `commit()` doesn't normally have to be called — a transaction will automatically commit when all
    /// outstanding requests have been satisfied and no new requests have been made.
    ///
    /// [Reference](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction/commit)
    pub async fn commit(self) -> Result<()> {
        let done = wait_request(&self.idb_transaction, Error::TransactionExecutionFailed);

        self.idb_transaction
            .commit()
            .map_err(Error::TransactionCommitFailed)?;

        done.await.map(|_| ())
    }

    /// Waits for a transaction to complete.
    pub async fn done(self) -> Result<()> {
        wait_request(self.idb_transaction, Error::TransactionExecutionFailed)
            .await
            .map(|_| ())
    }

    /// Returns a store in the transaction
    pub fn store(&self, store_name: &str) -> Result<Store> {
        Ok(Store::new(
            self.idb_transaction
                .object_store(store_name)
                .map_err(Error::ObjectStoreOpenFailed)?,
        ))
    }
}
