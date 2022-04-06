use std::collections::HashSet;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, IdbDatabase, IdbFactory, IdbOpenDbRequest, Window};

use crate::{request::wait_request, utils::set_panic_hook, Error, ObjectStore, Result, Rexie};

/// Builder for creating a new database.
pub struct RexieBuilder {
    name: String,
    version: Option<u32>,
    object_stores: Vec<ObjectStore>,
}

impl RexieBuilder {
    /// Creates a new database builder with given name.
    pub fn new(name: &str) -> Self {
        set_panic_hook();

        Self {
            name: name.to_owned(),
            version: None,
            object_stores: Vec::new(),
        }
    }

    /// Specify version of the database.
    pub fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    /// Add an object store to the database.
    pub fn add_object_store(mut self, object_store: ObjectStore) -> Self {
        self.object_stores.push(object_store);
        self
    }

    /// Build the database.
    pub async fn build(self) -> Result<Rexie> {
        let idb_open_request = get_idb_open_request(&self.name, self.version)?;
        let _upgrade_handler = set_upgrade_handler(&idb_open_request, self.object_stores);
        let db = wait_request(idb_open_request, Error::IndexedDbOpenFailed)
            .await?
            .unchecked_into();

        Ok(Rexie { db })
    }

    /// Delete the database.
    pub async fn delete(self) -> Result<()> {
        let factory = get_idb_factory()?;
        let idb_open_request = factory
            .delete_database(&self.name)
            .map_err(Error::IndexedDbOpenFailed)?;

        wait_request(idb_open_request, Error::IndexedDbDeleteFailed)
            .await
            .map(|_| ())
    }
}

fn get_window() -> Result<Window> {
    web_sys::window().ok_or(Error::WindowNotFound)
}

fn get_idb_factory() -> Result<IdbFactory> {
    let window = get_window()?;

    window
        .indexed_db()
        .map_err(Error::IndexedDbNotSupported)?
        .ok_or(Error::IndexedDbNotFound)
}

fn get_idb_open_request(name: &str, version: Option<u32>) -> Result<IdbOpenDbRequest> {
    let idb_factory = get_idb_factory()?;

    match version {
        Some(version) => idb_factory.open_with_u32(name, version),
        None => idb_factory.open(name),
    }
    .map_err(Error::IndexedDbOpenFailed)
}

fn set_upgrade_handler(
    idb_open_request: &IdbOpenDbRequest,
    object_stores: Vec<ObjectStore>,
) -> Closure<dyn FnMut(Event) -> std::result::Result<(), js_sys::Error>> {
    let upgrade_handler = Closure::once(move |event: Event| {
        upgrade_handler(event, object_stores).map_err(|err| js_sys::Error::new(&format!("{err:?}")))
    });

    idb_open_request.set_onupgradeneeded(Some(upgrade_handler.as_ref().unchecked_ref()));

    upgrade_handler
}

fn upgrade_handler(event: Event, object_stores: Vec<ObjectStore>) -> Result<()> {
    let mut store_names: HashSet<String> = object_stores.iter().map(|os| os.name.clone()).collect();

    let idb_open_request: IdbOpenDbRequest = event
        .target()
        .ok_or(Error::EventTargetNotFound)?
        .unchecked_into();

    let idb: IdbDatabase = idb_open_request
        .result()
        .map_err(Error::IndexedDbNotSupported)?
        .unchecked_into();

    for object_store in object_stores {
        object_store.create(&idb_open_request, &idb)?;
    }

    let db_store_names = idb.object_store_names();
    let mut stores_to_remove = Vec::new();

    for index in 0..db_store_names.length() {
        let db_store_name = db_store_names.get(index).unwrap_throw();

        if store_names.contains(&db_store_name) {
            store_names.remove(&db_store_name);
        } else {
            stores_to_remove.push(db_store_name);
        }
    }

    for store_name in stores_to_remove {
        idb.delete_object_store(&store_name).unwrap_throw();
    }

    Ok(())
}
