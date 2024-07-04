mod index;
mod store;

pub use self::{index::StoreIndex, store::Store};

use idb::Transaction as IdbTransaction;

use crate::{Error, Result, TransactionMode, TransactionResult};

/// Transaction on the database
pub struct Transaction {
    pub(crate) transaction: IdbTransaction,
}

impl Transaction {
    /// Returns mode of the transaction
    pub fn mode(&self) -> Result<TransactionMode> {
        self.transaction.mode().map_err(Into::into)
    }

    /// Returns names of all stores in the transaction
    pub fn store_names(&self) -> Vec<String> {
        self.transaction.store_names()
    }

    /// Aborts a transaction
    pub async fn abort(self) -> Result<()> {
        let result = self.transaction.abort()?.await?;

        if result.is_aborted() {
            Ok(())
        } else {
            Err(Error::TransactionAbortFailed)
        }
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
        let result = self.transaction.commit()?.await?;

        if result.is_committed() {
            Ok(())
        } else {
            Err(Error::TransactioncommitFailed)
        }
    }

    /// Waits for a transaction to complete.
    pub async fn done(self) -> Result<TransactionResult> {
        self.transaction.await.map_err(Into::into)
    }

    /// Returns a store in the transaction
    pub fn store(&self, store_name: &str) -> Result<Store> {
        self.transaction
            .object_store(store_name)
            .map(|object_store| Store { object_store })
            .map_err(Into::into)
    }
}
