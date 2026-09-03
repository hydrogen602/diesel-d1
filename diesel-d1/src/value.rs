use std::fmt;

use js_sys::{Array, ArrayBuffer, JsString, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug, Clone)]
pub struct D1Value(pub(crate) JsValue);

/// JS `Number.MAX_SAFE_INTEGER`
///
/// JS numbers are doubles, and
/// after this point, the minimum precision
/// is greater than one.
///
/// JS Example:
/// ```js
/// > 9007199254740993 === 9007199254740992
/// true
/// ```
const NUMBER_MAX_SAFE_INTEGER: i64 = 9007199254740991;

pub(crate) fn exceeds_js_safe_integer(value: i64) -> bool {
    value > NUMBER_MAX_SAFE_INTEGER || value < -NUMBER_MAX_SAFE_INTEGER
}

#[derive(Debug, thiserror::Error)]
pub enum IntError {
    #[error("Number is not a number")]
    NotANumber(D1Value),
    #[error("Number is not an integer: {0}")]
    NotAnInteger(f64),
    #[error("integer {0} is outside the Number.MAX_SAFE_INTEGER range")]
    NotASafeInteger(i64),
}

impl D1Value {
    pub(crate) fn read_string(&self) -> Option<String> {
        self.0.as_string()
    }
    /// JS numbers are always f64, this might cause precision issues when crossing boundaries
    pub(crate) fn read_number(&self) -> Option<f64> {
        self.0.as_f64()
    }

    pub(crate) fn read_integer(&self) -> Result<i64, IntError> {
        let number = self
            .read_number()
            .ok_or(IntError::NotANumber(self.clone()))?;

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(IntError::NotAnInteger(number).into());
        }
        let int = number as i64;
        if exceeds_js_safe_integer(int) {
            return Err(IntError::NotASafeInteger(int).into());
        }
        Ok(int)
    }

    pub(crate) fn read_blob(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
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
                    .ok_or_else(|| format!("D1 blob array element was not a number: {value:?}"))?;
                if !(0.0..=255.0).contains(&number) || number.fract() != 0.0 {
                    return Err(format!("D1 blob array element out of byte range: {number}").into());
                }
                bytes.push(number as u8);
            }
            return Ok(bytes);
        }
        Err(format!("D1 blob was not bytes (js type: {})", self.js_typeof()).into())
    }

    pub(crate) fn js_typeof(&self) -> String {
        self.0
            .js_typeof()
            .as_string()
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub(crate) fn js_to_string(&self) -> String {
        ToString::to_string(&JsString::from(self.0.clone()))
    }
}

impl fmt::Display for D1Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.js_to_string())
    }
}
