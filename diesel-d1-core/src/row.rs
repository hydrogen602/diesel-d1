use std::rc::Rc;

use diesel::row::{Field, PartialRow, Row, RowIndex, RowSealed};

use crate::{backend::D1Backend, value::D1Value};

/// One result row, with column names in SELECT order.
///
/// Invariant: the length of `column_names` and `fields` are the same,
/// and they are in the same order.
pub struct D1Row<'a> {
    column_names: Rc<[String]>,
    fields: Box<[D1Value<'a>]>,
}

impl<'a> D1Row<'a> {
    pub fn new(column_names: Rc<[String]>, fields: Box<[D1Value<'a>]>) -> Self {
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
unsafe impl<'a> Send for D1Row<'a> {}
unsafe impl<'a> Sync for D1Row<'a> {}

impl<'a> RowSealed for D1Row<'a> {}

impl<'stmt> Row<'stmt, D1Backend> for D1Row<'static> {
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
        let value = self.fields.get(index)?.as_ref();

        Some(D1Field { value, name })
    }

    fn partial_row(
        &self,
        range: std::ops::Range<usize>,
    ) -> diesel::row::PartialRow<'_, Self::InnerPartialRow> {
        PartialRow::new(self, range)
    }
}

impl RowIndex<usize> for D1Row<'_> {
    fn idx(&self, idx: usize) -> Option<usize> {
        if idx < self.fields.len() {
            Some(idx)
        } else {
            None
        }
    }
}

impl RowIndex<&str> for D1Row<'_> {
    fn idx(&self, field: &str) -> Option<usize> {
        self.column_names.iter().position(|name| name == field)
    }
}

pub struct D1Field<'a> {
    value: D1Value<'a>,
    name: &'a str,
}

impl<'stmt> Field<'stmt, D1Backend> for D1Field<'stmt> {
    fn field_name(&self) -> Option<&str> {
        Some(self.name)
    }

    fn value(&self) -> Option<D1Value<'_>> {
        if let D1Value::Null = self.value {
            None
        } else {
            Some(self.value.as_ref())
        }
    }
}
