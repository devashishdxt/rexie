use std::fmt;

use js_sys::Error as JsError;
use thiserror::Error;
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "js"))]
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "js")]
pub type Result<T> = std::result::Result<T, JsValue>;

#[derive(Debug, Error, Clone)]
pub struct Error {
    error_type: ErrorType,
    inner: Option<JsValue>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            None => write!(f, "{}", self.error_type),
            Some(ref inner) => write!(f, "{}: {}", self.error_type, option_display(inner)),
        }
    }
}

fn option_display(option: &JsValue) -> String {
    match option.as_string() {
        Some(s) => s,
        None => "".to_string(),
    }
}

impl Error {
    pub fn error_type(&self) -> ErrorType {
        self.error_type
    }

    pub(crate) fn set_inner(mut self, inner: JsValue) -> Self {
        self.inner = Some(inner);
        self
    }
}

impl From<ErrorType> for Error {
    fn from(error_type: ErrorType) -> Self {
        Self {
            error_type,
            inner: None,
        }
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ErrorType {
    #[error("error when receiving message from async channel")]
    AsyncChannelError,

    #[error("event target is none")]
    EventTargetNotFound,

    #[error("index creation failed")]
    IndexCreationFailed,

    #[error("index open failed")]
    IndexOpenFailed,

    #[error("indexed db error")]
    IndexedDBError,

    #[error("indexed db is none")]
    IndexedDBNotFound,

    #[error("indexed db not supported")]
    IndexedDBNotSupported,

    #[error("failed to open indexed db")]
    IndexedDBOpenFailed,

    #[error("key range error")]
    KeyRangeError,

    #[error("object store creation failed")]
    ObjectStoreCreationFailed,

    #[error("failed to open object store")]
    ObjectStoreOpenFailed,

    #[error("failed to execute db transaction")]
    TransactionExecutionFailed,

    #[error("failed to open db transaction")]
    TransactionOpenFailed,

    #[error("window object not found")]
    WindowNotFound,
}

impl ErrorType {
    pub(crate) fn into_error(self) -> Error {
        Error::from(self)
    }
}

impl From<ErrorType> for JsValue {
    fn from(error_type: ErrorType) -> Self {
        JsError::new(&error_type.to_string()).into()
    }
}

impl From<Error> for JsValue {
    fn from(error: Error) -> JsValue {
        match error.inner {
            Some(inner) => inner,
            None => error.error_type.into(),
        }
    }
}
