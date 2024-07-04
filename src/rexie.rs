use idb::Database;

use crate::{Result, RexieBuilder, Transaction, TransactionMode};

/// Rexie database (wrapper on top of indexed db)
#[derive(Debug)]
pub struct Rexie {
    pub(crate) database: Database,
}

impl Rexie {
    /// Creates a builder for database with given name
    pub fn builder(name: &str) -> RexieBuilder {
        RexieBuilder::new(name)
    }

    /// Returns name of the database
    pub fn name(&self) -> String {
        self.database.name()
    }

    /// Returns version of the database
    pub fn version(&self) -> Result<u32> {
        self.database.version().map_err(Into::into)
    }

    /// Returns names of all stores in the database
    pub fn store_names(&self) -> Vec<String> {
        self.database.store_names()
    }

    /// Creates a new transaction on the database
    pub fn transaction<T: AsRef<str>>(
        &self,
        store_names: &[T],
        mode: TransactionMode,
    ) -> Result<Transaction> {
        let transaction = self.database.transaction(store_names, mode)?;
        Ok(Transaction { transaction })
    }

    /// Closes the database
    pub fn close(self) {
        self.database.close();
    }

    /// Deletes a database
    pub async fn delete(name: &str) -> Result<()> {
        Self::builder(name).delete().await
    }
}
