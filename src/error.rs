use js_sys::Error as JsError;
use thiserror::Error;
use wasm_bindgen::prelude::*;

/// Result with `rexie::Error` as error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for `rexie` crate
#[derive(Debug, Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// Error when receiving message from async channel
    #[error("error when receiving message from async channel")]
    AsyncChannelError,

    /// Error when fetching DOM exception
    #[error("error when fetching DOM exception: {}", js_error_display(.0))]
    DomExceptionError(JsValue),

    /// DOM Exception is none
    #[error("dom exception is none")]
    DomExceptionNotFound,

    /// Event target is none
    #[error("event target is none")]
    EventTargetNotFound,

    /// Index creation failed
    #[error("index creation failed: {}", js_error_display(.0))]
    IndexCreationFailed(JsValue),

    /// Index open failed
    #[error("index open failed: {}", js_error_display(.0))]
    IndexOpenFailed(JsValue),

    /// Failed to delete indexed db
    #[error("failed to delete indexed db: {}", js_error_display(.0))]
    IndexedDbDeleteFailed(JsValue),

    /// Indexed db not found
    #[error("indexed db is none")]
    IndexedDbNotFound(JsValue),

    /// Indexed db not supported
    #[error("indexed db not supported: {}", js_error_display(.0))]
    IndexedDbNotSupported(JsValue),

    /// Failed to open indexed db
    #[error("failed to open indexed db: {}", js_error_display(.0))]
    IndexedDbOpenFailed(JsValue),

    /// Failed to execute indexed db request
    #[error("failed to execute indexed db request: {}", js_error_display(.0))]
    IndexedDbRequestError(JsValue),

    /// Failed to execute indexed db upgrade
    #[error("failed to execute indexed db upgrade: {}", js_error_display(.0))]
    IndexedDbUpgradeFailed(JsValue),

    /// Key range error
    #[error("key range error: {}", js_error_display(.0))]
    KeyRangeError(JsValue),

    /// Object store creation failed
    #[error("object store creation failed: {}", js_error_display(.0))]
    ObjectStoreCreationFailed(JsValue),

    /// Failed to open object store
    #[error("failed to open object store: {}", js_error_display(.0))]
    ObjectStoreOpenFailed(JsValue),

    /// Failed to commit indexed db transaction
    #[error("failed to commit indexed db transaction: {}", js_error_display(.0))]
    TransactionCommitFailed(JsValue),

    /// Failed to execute indexed db transaction
    #[error("failed to execute db transaction: {}", js_error_display(.0))]
    TransactionExecutionFailed(JsValue),

    /// Transaction is none
    #[error("transaction is none")]
    TransactionNotFound,

    /// failed to open db transaction
    #[error("failed to open db transaction: {}", js_error_display(.0))]
    TransactionOpenFailed(JsValue),

    /// Unexpected JS type
    #[error("unexpected js type")]
    UnexpectedJsType,

    /// window object is none
    #[error("window object is none")]
    WindowNotFound,
}

fn js_error_display(option: &JsValue) -> String {
    ToString::to_string(&JsError::from(option.clone()).to_string())
}

impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        match error {
            Error::AsyncChannelError => "AsyncChannelError".into(),
            Error::EventTargetNotFound => "EventTargetNotFound".into(),
            Error::IndexCreationFailed(js_value) => js_value,
            Error::IndexOpenFailed(js_value) => js_value,
            Error::IndexedDbNotFound(js_value) => js_value,
            Error::IndexedDbNotSupported(js_value) => js_value,
            Error::IndexedDbOpenFailed(js_value) => js_value,
            Error::IndexedDbUpgradeFailed(js_value) => js_value,
            Error::KeyRangeError(js_value) => js_value,
            Error::ObjectStoreCreationFailed(js_value) => js_value,
            Error::ObjectStoreOpenFailed(js_value) => js_value,
            Error::TransactionCommitFailed(js_value) => js_value,
            Error::TransactionExecutionFailed(js_value) => js_value,
            Error::TransactionOpenFailed(js_value) => js_value,
            Error::WindowNotFound => "WindowNotFound".into(),
            Error::DomExceptionError(js_value) => js_value,
            Error::DomExceptionNotFound => "DomExceptionNotFound".into(),
            Error::IndexedDbDeleteFailed(js_value) => js_value,
            Error::TransactionNotFound => "TransactionNotFound".into(),
            Error::IndexedDbRequestError(js_value) => js_value,
            Error::UnexpectedJsType => "UnxpectedJsType".into(),
        }
    }
}
