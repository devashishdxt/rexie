use js_sys::Array;
use wasm_bindgen::prelude::*;

pub(crate) enum KeyPath {
    String(String),
    Array(Vec<String>),
}

impl KeyPath {
    pub fn new_str(key_path: &str) -> Self {
        Self::String(key_path.to_owned())
    }

    pub fn new_array<'a>(key_path_array: impl IntoIterator<Item = &'a str>) -> Self {
        Self::Array(key_path_array.into_iter().map(ToOwned::to_owned).collect())
    }
}

impl From<KeyPath> for JsValue {
    fn from(key_path: KeyPath) -> Self {
        match key_path {
            KeyPath::String(key_path) => JsValue::from_str(&key_path),
            KeyPath::Array(key_path_array) => {
                let key_path = key_path_array
                    .iter()
                    .map(|s| JsValue::from_str(s))
                    .collect::<Array>();
                key_path.into()
            }
        }
    }
}
