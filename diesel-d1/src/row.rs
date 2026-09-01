use diesel::row::{Field, PartialRow, Row, RowIndex, RowSealed};
use wasm_bindgen::JsValue;

use crate::{backend::D1Backend, value::D1Value};

/// One result row, with column names in SELECT order.
///
/// Stored as `(name, value)` pairs so duplicate names (joins) stay positional
/// for [`Queryable`], while [`QueryableByName`] can look up the first match.
///
/// TODO: it might make sense to not own the keys
///   as currently every row has its own copy of the keys
///   either `Vec<(&str, JsValue)>` or a `key: Arc<[String]>`
pub struct D1Row {
    fields: Vec<(String, JsValue)>,
}

impl D1Row {
    pub(crate) fn from_named_values(fields: Vec<(String, JsValue)>) -> Self {
        Self { fields }
    }
}

// SAFETY: this is safe under WASM and workers because there's no threads and therefore no race conditions (at least memory ones)
unsafe impl Send for D1Row {}
unsafe impl Sync for D1Row {}

impl RowSealed for D1Row {}

impl<'stmt> Row<'stmt, D1Backend> for D1Row {
    type Field<'f>
        = D1Field<'f>
    where
        'stmt: 'f,
        Self: 'f;

    type InnerPartialRow = Self;

    fn field_count(&self) -> usize {
        self.fields.len()
    }

    fn get<'b, I>(&'b self, idx: I) -> Option<Self::Field<'b>>
    where
        'stmt: 'b,
        Self: diesel::row::RowIndex<I>,
    {
        let index = self.idx(idx)?;
        let (name, value) = self.fields.get(index)?;
        Some(D1Field {
            value: value.clone(),
            name,
        })
    }

    fn partial_row(
        &self,
        range: std::ops::Range<usize>,
    ) -> diesel::row::PartialRow<'_, Self::InnerPartialRow> {
        PartialRow::new(self, range)
    }
}

impl RowIndex<usize> for D1Row {
    fn idx(&self, idx: usize) -> Option<usize> {
        if idx < self.fields.len() {
            Some(idx)
        } else {
            None
        }
    }
}

impl RowIndex<&str> for D1Row {
    fn idx(&self, field: &str) -> Option<usize> {
        self.fields.iter().position(|(name, _)| name == field)
    }
}

pub struct D1Field<'a> {
    value: JsValue,
    name: &'a str,
}

impl<'stmt> Field<'stmt, D1Backend> for D1Field<'stmt> {
    fn field_name(&self) -> Option<&str> {
        Some(self.name)
    }

    fn value(&self) -> Option<D1Value> {
        if self.value.is_null() || self.value.is_undefined() {
            None
        } else {
            Some(D1Value(self.value.clone()))
        }
    }
}
