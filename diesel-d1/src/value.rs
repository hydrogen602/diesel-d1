use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};

pub struct D1Value(pub JsValue);

impl D1Value {
    pub(crate) fn read_string(&self) -> String {
        self.0.as_string().unwrap()
    }

    pub(crate) fn read_bool(&self) -> bool {
        self.0.as_bool().unwrap()
    }

    /// JS numbers are always f64, this might cause precision issues when crossing boundaries
    pub(crate) fn read_number(&self) -> f64 {
        self.0.as_f64().unwrap()
    }

    pub(crate) fn check_null(&self) -> bool {
        // not sure if undefined works
        self.0.is_null() || self.0.is_undefined()
    }

    pub(crate) fn read_blob(&self) -> Vec<u8> {
        let x = self
            .0
            .dyn_ref::<Uint8Array>()
            .expect("JSValue is not uint8arrary");
        x.to_vec()
    }
}
