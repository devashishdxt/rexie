use std::collections::HashSet;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, IdbDatabase, IdbFactory, IdbOpenDbRequest, Window};

use crate::{
    observer::ResultObserver, utils::set_panic_hook, ErrorType, ObjectStore, Result, Rexie,
};

#[cfg_attr(feature = "js", wasm_bindgen)]
pub struct RexieBuilder {
    name: String,
    version: Option<u32>,
    object_stores: Vec<ObjectStore>,
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl RexieBuilder {
    #[cfg_attr(feature = "js", wasm_bindgen(constructor))]
    pub fn new(name: &str) -> Self {
        set_panic_hook();

        Self {
            name: name.to_owned(),
            version: None,
            object_stores: Vec::new(),
        }
    }

    pub fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    #[cfg_attr(feature = "js", wasm_bindgen(js_name = "addObjectStore"))]
    pub fn add_object_store(mut self, object_store: ObjectStore) -> Self {
        self.object_stores.push(object_store);
        self
    }

    pub async fn build(self) -> Result<Rexie> {
        let idb_open_request = get_idb_open_request(&self.name, self.version)?;

        let _upgrade_handler = set_upgrade_handler(&idb_open_request, self.object_stores);

        let observer = ResultObserver::new((), |event| {
            let target = event.target().unwrap_throw();
            let request: &IdbOpenDbRequest = AsRef::<JsValue>::as_ref(&target).unchecked_ref();

            #[allow(clippy::useless_conversion)]
            match request.error() {
                Ok(Some(exception)) => Err(ErrorType::IndexedDBOpenFailed
                    .into_error()
                    .set_inner(exception.into())
                    .into()),
                Ok(None) => Err(ErrorType::IndexedDBError.into()),
                Err(error) => Err(ErrorType::IndexedDBError
                    .into_error()
                    .set_inner(error)
                    .into()),
            }
        });

        idb_open_request.set_onsuccess(Some(observer.get_success_callback()));
        idb_open_request.set_onerror(Some(observer.get_error_callback()));

        observer.finish().await?;

        let db = idb_open_request
            .result()
            .map_err(|js_value| {
                ErrorType::IndexedDBNotFound
                    .into_error()
                    .set_inner(js_value)
            })?
            .unchecked_into();

        Ok(Rexie { db })
    }
}

fn get_window() -> Result<Window> {
    web_sys::window()
        .ok_or_else(|| ErrorType::WindowNotFound.into_error())
        .map_err(Into::into)
}

fn get_idb_factory() -> Result<IdbFactory> {
    let window = get_window()?;

    window
        .indexed_db()
        .map_err(|js_value| {
            ErrorType::IndexedDBNotSupported
                .into_error()
                .set_inner(js_value)
        })?
        .ok_or_else(|| ErrorType::IndexedDBNotSupported.into_error())
        .map_err(Into::into)
}

fn get_idb_open_request(name: &str, version: Option<u32>) -> Result<IdbOpenDbRequest> {
    let idb_factory = get_idb_factory()?;

    match version {
        Some(version) => idb_factory.open_with_u32(name, version),
        None => idb_factory.open(name),
    }
    .map_err(|js_value| {
        ErrorType::IndexedDBOpenFailed
            .into_error()
            .set_inner(js_value)
    })
    .map_err(Into::into)
}

fn set_upgrade_handler(
    idb_open_request: &IdbOpenDbRequest,
    object_stores: Vec<ObjectStore>,
) -> Closure<dyn FnMut(Event)> {
    let upgrade_handler = Closure::once(move |event: Event| {
        upgrade_handler(event, object_stores).unwrap_throw();
    });

    idb_open_request.set_onupgradeneeded(Some(upgrade_handler.as_ref().unchecked_ref()));

    upgrade_handler
}

fn upgrade_handler(event: Event, object_stores: Vec<ObjectStore>) -> Result<()> {
    let mut store_names: HashSet<String> = object_stores.iter().map(|os| os.name.clone()).collect();

    let idb_open_request: IdbOpenDbRequest = event
        .target()
        .ok_or_else(|| ErrorType::EventTargetNotFound.into_error())?
        .unchecked_into();

    let idb: IdbDatabase = idb_open_request
        .result()
        .map_err(|js_value| {
            ErrorType::IndexedDBNotFound
                .into_error()
                .set_inner(js_value)
        })?
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
