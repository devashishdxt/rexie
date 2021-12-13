use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::IdbRequest;

use crate::{observer::ResultObserver, ErrorType, Result};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub async fn wait_request(idb_request: &IdbRequest) -> Result<()> {
    let observer = ResultObserver::new((), |event| {
        let target = event.target().unwrap_throw();
        let request: &IdbRequest = AsRef::<JsValue>::as_ref(&target).unchecked_ref();

        #[allow(clippy::useless_conversion)]
        match request.error() {
            Ok(Some(exception)) => Err(ErrorType::IndexedDBError
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

    idb_request.set_onsuccess(Some(observer.get_success_callback()));
    idb_request.set_onerror(Some(observer.get_error_callback()));

    observer.finish().await
}
