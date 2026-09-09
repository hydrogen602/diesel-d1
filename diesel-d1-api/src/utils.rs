use std::fmt::{Debug, Display};

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
