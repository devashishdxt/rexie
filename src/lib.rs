//! ## Usage
//!
//! ```rust,ignore
//! let rexie = Rexie::builder("backend")
//!     .version(1)
//!     .add_object_store(
//!         ObjectStore::new("employees")
//!             .key_path("id")
//!             .add_index(Index::new("email", "email").unique(true)),
//!     )
//!     .build()
//!     .await?;
//!
//! web_sys::console::log_1(&rexie.name().into());
//! ```
mod error;
mod index;
mod key_range;
mod object_store;
mod observer;
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
    error::{Error, ErrorType, Result},
    index::Index,
    key_range::KeyRange,
    object_store::ObjectStore,
    rexie::Rexie,
    rexie_builder::RexieBuilder,
    transaction::{Store, StoreIndex, Transaction, TransactionMode},
};

// #[cfg(feature = "js-test")]
// use wasm_bindgen::prelude::*;

// #[cfg(feature = "js-test")]
// #[wasm_bindgen]
// pub async fn test() -> std::result::Result<(), JsValue> {
//     let rexie = Rexie::builder("backend")
//         .version(1)
//         .add_object_store(ObjectStore::new("employees"))
//         .build()
//         .await?;

//     let transaction =
//         rexie.transaction(vec!["employees".to_string()], TransactionMode::ReadWrite)?;
//     let store = transaction.store("employees")?;

//     store.add(JsValue::from(3), JsValue::from("the"))?;
//     web_sys::console::log_1(&store.count().await?.into());

//     store.add(JsValue::from(4), JsValue::from("rock"))?;
//     web_sys::console::log_1(&store.count().await?.into());

//     transaction.finish().await?;

//     web_sys::console::log_1(&rexie.name().into());

//     rexie.close();

//     Ok(())
// }
