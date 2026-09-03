use js_sys::{Array, ArrayBuffer, Uint8Array};
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

    pub(crate) fn read_blob(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(bytes) = self.0.dyn_ref::<Uint8Array>() {
            return Ok(bytes.to_vec());
        }
        if ArrayBuffer::instanceof(&self.0) {
            return Ok(Uint8Array::new(&self.0).to_vec());
        }
        if let Some(arr) = self.0.dyn_ref::<Array>() {
            let mut bytes = Vec::with_capacity(arr.length() as usize);
            for value in arr.iter() {
                let number = value
                    .as_f64()
                    .ok_or_else(|| format!("D1 blob array element was not a number: {value:?}"))?;
                if !(0.0..=255.0).contains(&number) || number.fract() != 0.0 {
                    return Err(format!("D1 blob array element out of byte range: {number}").into());
                }
                bytes.push(number as u8);
            }
            return Ok(bytes);
        }
        Err(format!(
            "D1 blob was not bytes (js type: {})",
            self.0
                .js_typeof()
                .as_string()
                .unwrap_or_else(|| "unknown".to_string())
        )
        .into())
    }
}
