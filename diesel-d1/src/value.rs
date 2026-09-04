use std::fmt;

use diesel_d1_core::value::JsonLikeValue;
use js_sys::{Array, ArrayBuffer, JsString, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug, Clone)]
pub struct D1Value(pub(crate) JsValue);

impl JsonLikeValue for D1Value {
    type BlobError = BlobError;

    fn js_to_string(&self) -> String {
        ToString::to_string(&JsString::from(self.0.clone()))
    }

    fn read_string(&self) -> Option<String> {
        self.0.as_string()
    }
    /// JS numbers are always f64, this might cause precision issues when crossing boundaries
    fn read_number(&self) -> Option<f64> {
        self.0.as_f64()
    }

    fn read_blob(&self) -> Result<Vec<u8>, BlobError> {
        if let Some(bytes) = self.0.dyn_ref::<Uint8Array>() {
            return Ok(bytes.to_vec());
        }
        if ArrayBuffer::instanceof(&self.0) {
            return Ok(Uint8Array::new(&self.0).to_vec());
        }
        // And... D1 returns a blob as an array of JS numbers
        if let Some(arr) = self.0.dyn_ref::<Array>() {
            let mut bytes = Vec::with_capacity(arr.length() as usize);
            for value in arr.iter() {
                let number = value
                    .as_f64()
                    .ok_or(BlobError::ElementNotANumber(D1Value(value).to_string()))?;
                if !(0.0..=255.0).contains(&number) || number.fract() != 0.0 {
                    return Err(BlobError::ElementOutOfByteRange(number));
                }
                bytes.push(number as u8);
            }
            return Ok(bytes);
        }
        Err(BlobError::NotABlob(self.to_string()))
    }
}

impl D1Value {
    pub(crate) fn js_typeof(&self) -> String {
        self.0
            .js_typeof()
            .as_string()
            .unwrap_or_else(|| "unknown".to_string())
    }
}

impl fmt::Display for D1Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.js_to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("Value is not a blob")]
    NotABlob(String),
    #[error("Blob array element was not a number: {0:?}")]
    ElementNotANumber(String),
    #[error("Blob array element out of byte range: {0}")]
    ElementOutOfByteRange(f64),
}
