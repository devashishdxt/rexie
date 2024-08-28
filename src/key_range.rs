use idb::{KeyRange as IdbKeyRange, Query};
use wasm_bindgen::JsValue;

use crate::Error;

/// Represents a continuous interval over some data type that is used for keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRange {
    inner: IdbKeyRange,
}

impl KeyRange {
    /// Returns a new [`KeyRange`] spanning only key.
    pub fn only(value: &JsValue) -> Result<Self, Error> {
        IdbKeyRange::only(value).map(Into::into).map_err(Into::into)
    }

    /// Returns a new [`KeyRange`] spanning from lower to upper. If `lower_open` is true, `lower` is not included in the
    /// range. If `upper_open` is true, `upper` is not included in the range.
    pub fn bound(
        lower: &JsValue,
        upper: &JsValue,
        lower_open: Option<bool>,
        upper_open: Option<bool>,
    ) -> Result<Self, Error> {
        IdbKeyRange::bound(lower, upper, lower_open, upper_open)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns a new [`KeyRange`] starting at key with no upper bound. If `lower_open` is true, key is not included in
    /// the range.
    pub fn lower_bound(lower: &JsValue, lower_open: Option<bool>) -> Result<Self, Error> {
        IdbKeyRange::lower_bound(lower, lower_open)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns a new [`KeyRange`] with no lower bound and ending at key. If `upper_open` is true, key is not included
    /// in the range.
    pub fn upper_bound(upper: &JsValue, upper_open: Option<bool>) -> Result<Self, Error> {
        IdbKeyRange::upper_bound(upper, upper_open)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Returns the range’s lower bound, or undefined if none.
    pub fn lower(&self) -> Result<JsValue, Error> {
        self.inner.lower().map_err(Into::into)
    }

    /// Returns the range’s upper bound, or undefined if none.
    pub fn upper(&self) -> Result<JsValue, Error> {
        self.inner.upper().map_err(Into::into)
    }

    /// Returns the range’s lower open flag.
    pub fn lower_open(&self) -> bool {
        self.inner.lower_open()
    }

    /// Returns the range’s upper open flag.
    pub fn upper_open(&self) -> bool {
        self.inner.upper_open()
    }

    /// Returns true if key is included in the range, and false otherwise.
    pub fn includes(&self, value: &JsValue) -> Result<bool, Error> {
        self.inner.includes(value).map_err(Into::into)
    }
}

impl From<IdbKeyRange> for KeyRange {
    fn from(inner: IdbKeyRange) -> Self {
        Self { inner }
    }
}

impl From<KeyRange> for Query {
    fn from(key_range: KeyRange) -> Self {
        key_range.inner.into()
    }
}
