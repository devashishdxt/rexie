//! Rexie is an easy-to-use, futures based wrapper around IndexedDB that compiles to webassembly.
//!
//! # Usage
//!
//! To use Rexie, you need to add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rexie = "0.4"
//! ```
//!
//! ## Example
//!
//! To create a new database, you can use the [`Rexie::builder`] method:
//!
//! ```rust
//! use rexie::*;
//!
//! async fn build_database() -> Result<Rexie> {
//!    // Create a new database
//!    let rexie = Rexie::builder("test")
//!        // Set the version of the database to 1.0
//!        .version(1)
//!        // Add an object store named `employees`
//!        .add_object_store(
//!            ObjectStore::new("employees")
//!                // Set the key path to `id`
//!                .key_path("id")
//!                // Enable auto increment
//!                .auto_increment(true)
//!                // Add an index named `email` with the key path `email` with unique enabled
//!                .add_index(Index::new("email", "email").unique(true)),
//!        )
//!        // Build the database
//!        .build()
//!        .await?;
//!
//!     // Check basic details of the database
//!     assert_eq!(rexie.name(), "test");
//!     assert_eq!(rexie.version(), 1.0);
//!     assert_eq!(rexie.store_names(), vec!["employees"]);
//!
//!     Ok(rexie)
//! }
//! ```
//!
//! To add an employee, you can use the [`Store::add`] method after creating a [`Transaction`]:
//!
//! ```rust
//! use rexie::*;
//!
//! async fn add_employee(rexie: &Rexie, name: &str, email: &str) -> Result<u32> {
//!     // Create a new read-write transaction
//!     let transaction = rexie.transaction(&["employees"], TransactionMode::ReadWrite)?;
//!     
//!     // Get the `employees` store
//!     let employees = transaction.store("employees")?;
//!     
//!     // Create an employee
//!     let employee = serde_json::json!({
//!         "name": name,
//!         "email": email,
//!     });
//!     // Convert it to `JsValue`
//!     let employee = serde_wasm_bindgen::to_value(&employee).unwrap();
//!
//!     // Add the employee to the store
//!     let employee_id = employees.add(&employee, None).await?;
//!     
//!     // Waits for the transaction to complete
//!     transaction.done().await?;
//!
//!     // Return the employee id
//!     Ok(num_traits::cast(employee_id.as_f64().unwrap()).unwrap())
//! }
//! ```
//!
//! To get an employee, you can use the [`Store::get`] method after creating a [`Transaction`]:
//!
//! ```rust
//! use rexie::*;
//!
//! async fn get_employee(rexie: &Rexie, id: u32) -> Result<Option<serde_json::Value>> {
//!     // Create a new read-only transaction
//!     let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly)?;
//!     
//!     // Get the `employees` store
//!     let employees = transaction.store("employees")?;
//!     
//!     // Get the employee
//!     let employee = employees.get(&id.into()).await?;
//!
//!     // Convert it to `serde_json::Value` from `JsValue`
//!     let employee: Option<serde_json::Value> = serde_wasm_bindgen::from_value(employee).unwrap();
//!
//!     // Return the employee
//!     Ok(employee)
//! }
//! ```
mod direction;
mod error;
mod index;
mod key_range;
mod object_store;
mod request;
mod rexie;
mod rexie_builder;
mod transaction;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub use self::{
    direction::Direction,
    error::{Error, Result},
    index::Index,
    key_range::KeyRange,
    object_store::ObjectStore,
    rexie::Rexie,
    rexie_builder::RexieBuilder,
    transaction::{Store, StoreIndex, Transaction, TransactionMode},
};
