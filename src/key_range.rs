#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::IdbKeyRange;

use crate::{ErrorType, Result};

#[cfg_attr(feature = "js", wasm_bindgen)]
pub struct KeyRange {
    idb_key_range: IdbKeyRange,
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl KeyRange {
    #[cfg_attr(feature = "js", wasm_bindgen(js_name = "lowerBound"))]
    pub fn lower_bound(lower_bound: &JsValue, lower_open: bool) -> Result<KeyRange> {
        let idb_key_range = IdbKeyRange::lower_bound_with_open(lower_bound, lower_open)
            .map_err(|js_value| ErrorType::KeyRangeError.into_error().set_inner(js_value))?;

        Ok(KeyRange { idb_key_range })
    }

    #[cfg_attr(feature = "js", wasm_bindgen(js_name = "upperBound"))]
    pub fn upper_bound(upper_bound: &JsValue, upper_open: bool) -> Result<KeyRange> {
        let idb_key_range = IdbKeyRange::upper_bound_with_open(upper_bound, upper_open)
            .map_err(|js_value| ErrorType::KeyRangeError.into_error().set_inner(js_value))?;

        Ok(KeyRange { idb_key_range })
    }

    pub fn bound(
        lower: &JsValue,
        upper: &JsValue,
        lower_open: bool,
        upper_open: bool,
    ) -> Result<KeyRange> {
        let idb_key_range =
            IdbKeyRange::bound_with_lower_open_and_upper_open(lower, upper, lower_open, upper_open)
                .map_err(|js_value| ErrorType::KeyRangeError.into_error().set_inner(js_value))?;

        Ok(Self { idb_key_range })
    }

    pub fn only(value: &JsValue) -> Result<KeyRange> {
        let idb_key_range = IdbKeyRange::only(value)
            .map_err(|js_value| ErrorType::KeyRangeError.into_error().set_inner(js_value))?;

        Ok(Self { idb_key_range })
    }
}

impl AsRef<JsValue> for KeyRange {
    fn as_ref(&self) -> &JsValue {
        self.idb_key_range.as_ref()
    }
}
