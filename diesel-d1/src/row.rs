use std::rc::Rc;

use diesel::row::{Field, PartialRow, Row, RowIndex, RowSealed};
use wasm_bindgen::JsValue;

use crate::{backend::D1Backend, value::D1Value};

/// One result row, with column names in SELECT order.
///
/// Invariant: the length of `column_names` and `fields` are the same,
/// and they are in the same order.
pub struct D1Row {
    column_names: Rc<[String]>,
    fields: Box<[D1Value]>,
}

impl D1Row {
    pub(crate) fn new(column_names: Rc<[String]>, fields: Box<[D1Value]>) -> Self {
        debug_assert_eq!(
            column_names.len(),
            fields.len(),
            "column_names and fields must have the same length"
        );
        Self {
            column_names,
            fields,
        }
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

        let name = self.column_names.get(index)?.as_str();
        let D1Value(value) = self.fields.get(index)?;

        Some(D1Field { value, name })
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
        self.column_names.iter().position(|name| name == field)
    }
}

pub struct D1Field<'a> {
    value: &'a JsValue,
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
