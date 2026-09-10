use std::fmt::{Debug, Display};

/// A string that is not shown when displayed or debugged.
///
/// To ensure sensitive info isn't accidentally leaked in logs or errors.
pub struct NoShowString(String);

impl NoShowString {
    pub fn new(string: String) -> Self {
        Self(string)
    }

    pub(crate) fn get_str(&self) -> &str {
        &self.0
    }
}

impl Display for NoShowString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<hidden>")
    }
}

impl Debug for NoShowString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<hidden>")
    }
}
