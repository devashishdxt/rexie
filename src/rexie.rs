use js_sys::Array;
use wasm_bindgen::prelude::*;
use web_sys::IdbDatabase;

use crate::{Error, Result, RexieBuilder, Transaction, TransactionMode};

/// Rexie database (wrapper on top of indexed db)
pub struct Rexie {
    pub(crate) db: IdbDatabase,
}

impl Rexie {
    /// Creates a builder for database with given name
    pub fn builder(name: &str) -> RexieBuilder {
        RexieBuilder::new(name)
    }

    /// Returns name of the database
    pub fn name(&self) -> String {
        self.db.name()
    }

    /// Returns version of the database
    pub fn version(&self) -> f64 {
        self.db.version()
    }

    /// Returns names of all stores in the database
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

    /// Creates a new transaction on the database
    pub fn transaction<T: AsRef<str>>(
        &self,
        store_names: &[T],
        mode: TransactionMode,
    ) -> Result<Transaction> {
        let store_names: Array = store_names
            .iter()
            .map(|s| JsValue::from(s.as_ref()))
            .collect();

        let idb_transaction = self
            .db
            .transaction_with_str_sequence_and_mode(&store_names, mode.into())
            .map_err(Error::TransactionOpenFailed)?;

        Ok(Transaction { idb_transaction })
    }

    /// Closes the database
    pub fn close(self) {
        self.db.close();
    }

    /// Deletes a database
    pub async fn delete(name: &str) -> Result<()> {
        Self::builder(name).delete().await
    }
}
