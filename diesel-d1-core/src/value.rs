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
