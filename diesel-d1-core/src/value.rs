/// Values like a JSON value.
///
/// Can either be from JSON or a `JsValue`.
pub trait JsonLikeValue {
    type BlobError: std::error::Error;

    /// Convert to an informative string.
    /// - `JsValue` should use JS' `.ToString()`.
    /// - JSON should serialize.
    fn js_to_string(&self) -> String;

    /// Read a string from the value.
    fn read_string(&self) -> Option<String>;

    /// Read a number from the value.
    fn read_number(&self) -> Option<f64>;

    /// Read a boolean from the value.
    fn read_boolean(&self) -> Option<bool> {
        let int = self.read_integer().ok()?;
        match int {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    /// Read an integer from the value.
    fn read_integer(&self) -> Result<i64, IntError> {
        let number = self
            .read_number()
            .ok_or(IntError::NotANumber(self.js_to_string()))?;

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(IntError::NotAnInteger(number));
        }
        let int = number as i64;
        if exceeds_js_safe_integer(int) {
            return Err(IntError::UnsafeInteger(int));
        }
        Ok(int)
    }

    /// Read a blob from the value.
    fn read_blob(&self) -> Result<Vec<u8>, Self::BlobError>;
}

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

pub fn exceeds_js_safe_integer(value: i64) -> bool {
    !(-NUMBER_MAX_SAFE_INTEGER..=NUMBER_MAX_SAFE_INTEGER).contains(&value)
}

#[derive(Debug, thiserror::Error)]
pub enum IntError {
    #[error("Value is not a number")]
    NotANumber(String),
    #[error("Number is not an integer: {0}")]
    NotAnInteger(f64),
    #[error("integer {0} is outside the Number.MAX_SAFE_INTEGER range")]
    UnsafeInteger(i64),
}

/// What an SQLite & JSONs value can be.
///
/// This is what one value in a D1 row would be returned as.
#[derive(Debug)]
pub enum D1Value<'a> {
    Null,
    Number(f64),
    String(Box<str>),
    StringRef(&'a str),
    Blob(Box<[u8]>),
    BlobRef(&'a [u8]),
}

impl<'a> D1Value<'a> {
    /// No-clone way to get another D1Value
    pub fn as_ref(&'a self) -> D1Value<'a> {
        match self {
            D1Value::Null => D1Value::Null,
            D1Value::Number(number) => D1Value::Number(*number),
            D1Value::String(string) => D1Value::StringRef(string.as_ref()),
            D1Value::StringRef(string) => D1Value::StringRef(string),
            D1Value::Blob(blob) => D1Value::BlobRef(blob.as_ref()),
            D1Value::BlobRef(blob) => D1Value::BlobRef(blob),
        }
    }
}

impl JsonLikeValue for D1Value<'_> {
    type BlobError = BlobError;

    fn js_to_string(&self) -> String {
        match self {
            D1Value::Null => "null".to_string(),
            D1Value::Number(number) => number.to_string(),
            D1Value::String(string) => string.to_string(),
            D1Value::StringRef(string) => string.to_string(),
            D1Value::Blob(blob) => format!("blob of length {}", blob.len()),
            D1Value::BlobRef(blob) => format!("blob of length {}", blob.len()),
        }
    }

    fn read_string(&self) -> Option<String> {
        match self {
            D1Value::String(string) => Some(string.to_string()),
            D1Value::StringRef(string) => Some(string.to_string()),
            _ => None,
        }
    }

    fn read_number(&self) -> Option<f64> {
        match self {
            D1Value::Number(number) => Some(*number),
            D1Value::String(string) => string.parse().ok(),
            D1Value::StringRef(string) => string.parse().ok(),
            _ => None,
        }
    }

    fn read_blob(&self) -> Result<Vec<u8>, Self::BlobError> {
        match self {
            D1Value::Blob(blob) => Ok(blob.to_vec()),
            D1Value::BlobRef(blob) => Ok(blob.to_vec()),
            _ => Err(BlobError::NotABlob {
                typeof_: stringify!(D1Value).to_string(),
                to_string: self.js_to_string(),
            }),
        }
    }
}

pub type D1ValueOwned = D1Value<'static>;

#[cfg(feature = "worker")]
pub use worker_impls::{BlobError, NotConvertibleToD1ValueError, js_to_string, js_typeof};

#[cfg(feature = "worker")]
mod worker_impls {
    use js_sys::{Array, ArrayBuffer, JsString, Uint8Array};
    use wasm_bindgen::JsCast;

    use super::*;

    #[derive(Debug, thiserror::Error)]
    pub enum BlobError {
        #[error("Value is not a blob: typeof: {typeof_}, to_string: {to_string:?}")]
        NotABlob { typeof_: String, to_string: String },
        #[error("Blob array element was not a number: typeof: {typeof_}, to_string: {to_string:?}")]
        ElementNotANumber { typeof_: String, to_string: String },
        #[error("Blob array element out of byte range: {0}")]
        ElementOutOfByteRange(f64),
    }

    #[derive(Debug, thiserror::Error)]
    #[error(
        "Value is not convertible to a D1/SQLite value: typeof: {typeof_}, to_string: {to_string:?}. Tried null, undefined, string, number, boolean, blob."
    )]
    pub struct NotConvertibleToD1ValueError {
        pub typeof_: String,
        pub to_string: String,
    }

    impl<'a> TryFrom<wasm_bindgen::JsValue> for D1Value<'a> {
        type Error = NotConvertibleToD1ValueError;

        fn try_from(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
            if value.is_null_or_undefined() {
                Ok(D1Value::Null)
            } else if let Some(string) = value.as_string() {
                Ok(D1Value::String(string.into()))
            } else if let Some(number) = value.as_f64() {
                // js doesn't have an integer type so f64 also handles those.
                Ok(D1Value::Number(number))
            } else if let Some(bool) = value.as_bool() {
                // SQLite doesn't have a boolean type, and js doesn't have an integer type,
                // so we convert all to f64.
                Ok(D1Value::Number(i32::from(bool) as f64))
            } else if let Ok(bytes) = read_blob(&value) {
                Ok(D1Value::Blob(bytes.into()))
            } else {
                Err(NotConvertibleToD1ValueError {
                    typeof_: js_typeof(&value),
                    to_string: js_to_string(value),
                })
            }
        }
    }

    fn read_blob(value: &wasm_bindgen::JsValue) -> Result<Vec<u8>, BlobError> {
        if let Some(bytes) = value.dyn_ref::<Uint8Array>() {
            return Ok(bytes.to_vec());
        }
        if ArrayBuffer::instanceof(value) {
            return Ok(Uint8Array::new(value).to_vec());
        }
        // And... D1 returns a blob as an array of JS numbers
        if let Some(arr) = value.dyn_ref::<Array>() {
            let mut bytes = Vec::with_capacity(arr.length() as usize);
            for value in arr.iter() {
                let number = value.as_f64().ok_or(BlobError::ElementNotANumber {
                    typeof_: js_typeof(&value),
                    to_string: js_to_string(value),
                })?;
                if !(0.0..=255.0).contains(&number) || number.fract() != 0.0 {
                    return Err(BlobError::ElementOutOfByteRange(number));
                }
                bytes.push(number as u8);
            }
            return Ok(bytes);
        }
        Err(BlobError::NotABlob {
            typeof_: js_typeof(value),
            to_string: js_to_string(value.clone()),
        })
    }

    pub fn js_typeof(value: &wasm_bindgen::JsValue) -> String {
        value
            .js_typeof()
            .as_string()
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// JsString wants a JsValue, not a reference.
    pub fn js_to_string(value: wasm_bindgen::JsValue) -> String {
        JsString::from(value).into()
    }
}
