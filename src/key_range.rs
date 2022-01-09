use wasm_bindgen::prelude::*;
use web_sys::IdbKeyRange;

use crate::{Error, Result};

/// A key range.
pub struct KeyRange {
    idb_key_range: IdbKeyRange,
}

impl KeyRange {
    /// Creates a new key range with given lower bound. The lower bound is inclusive if `lower_open` is false and
    /// exclusive if `lower_open` is true.
    pub fn lower_bound(lower_bound: &JsValue, lower_open: bool) -> Result<KeyRange> {
        let idb_key_range = IdbKeyRange::lower_bound_with_open(lower_bound, lower_open)
            .map_err(Error::KeyRangeError)?;

        Ok(KeyRange { idb_key_range })
    }

    /// Creates a new key range with given upper bound. The upper bound is inclusive if `upper_open` is false and
    /// exclusive if `upper_open` is true.
    pub fn upper_bound(upper_bound: &JsValue, upper_open: bool) -> Result<KeyRange> {
        let idb_key_range = IdbKeyRange::upper_bound_with_open(upper_bound, upper_open)
            .map_err(Error::KeyRangeError)?;

        Ok(KeyRange { idb_key_range })
    }

    /// Creates a new key range with given lower and upper bound. The lower bound is inclusive if `lower_open` is false
    /// and exclusive if `lower_open` is true. The upper bound is inclusive if `upper_open` is false and exclusive if
    /// `upper_open` is true.
    pub fn bound(
        lower: &JsValue,
        upper: &JsValue,
        lower_open: bool,
        upper_open: bool,
    ) -> Result<KeyRange> {
        let idb_key_range =
            IdbKeyRange::bound_with_lower_open_and_upper_open(lower, upper, lower_open, upper_open)
                .map_err(Error::KeyRangeError)?;

        Ok(Self { idb_key_range })
    }

    /// Creates a new key range that matches with only one value.
    pub fn only(value: &JsValue) -> Result<KeyRange> {
        let idb_key_range = IdbKeyRange::only(value).map_err(Error::KeyRangeError)?;

        Ok(Self { idb_key_range })
    }
}

impl AsRef<JsValue> for KeyRange {
    fn as_ref(&self) -> &JsValue {
        self.idb_key_range.as_ref()
    }
}
