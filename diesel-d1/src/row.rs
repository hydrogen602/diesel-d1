use diesel::row::{Field, PartialRow, Row, RowIndex, RowSealed};
use wasm_bindgen::{JsCast, JsValue};

use crate::{backend::D1Backend, value::D1Value};

// pub struct D1Row {
//     _js_obj: Rc<RefCell<JsValue>>,
//     field_vec: Vec<String>,
// }

/// FIXME: Prob not performant but it's a start
///
/// Maybe it makes sense converting this to a Map in rust?
pub struct D1Row(pub js_sys::Object);

// SAFETY: this is safe under WASM and workers because there's no threads and therefore no race conditions (at least memory ones)
unsafe impl Send for D1Row {}
unsafe impl Sync for D1Row {}

// impl D1Row {
//     pub fn new(js_value: JsValue, field_vec: Vec<String>) -> Self {
//         Self {
//             // again
//             _js_obj: Rc::new(RefCell::new(js_value)),
//             field_vec,
//         }
//     }
// }

impl RowSealed for D1Row {}

impl<'stmt> Row<'stmt, D1Backend> for D1Row {
    type Field<'f>
        = D1Field
    where
        'stmt: 'f,
        Self: 'f;

    type InnerPartialRow = Self;

    fn field_count(&self) -> usize {
        js_sys::Object::keys(&self.0).length() as usize
    }

    fn get<'b, I>(&'b self, idx: I) -> Option<Self::Field<'b>>
    where
        'stmt: 'b,
        Self: diesel::row::RowIndex<I>,
    {
        let index = self.idx(idx)?;
        let entry = js_sys::Object::entries(&self.0).get(index as u32);
        if entry.is_undefined() {
            None
        } else {
            let entry = entry.dyn_into::<js_sys::Array>().unwrap(); // Object.entry returns an array of [key, value]
            let key = entry.get(0).as_string().unwrap(); // FIXME: could this be a number?
            let value = entry.get(1);
            Some(D1Field { value, name: key })
        }
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
        if idx < js_sys::Object::keys(&self.0).length() as usize {
            Some(idx)
        } else {
            None
        }
    }
}
// TODO(lduarte): it's not efficient to do it like this for now ahah, but JS
impl RowIndex<&str> for D1Row {
    fn idx(&self, field: &str) -> Option<usize> {
        let keys = js_sys::Object::keys(&self.0);
        keys.iter().position(|key| key == field)
    }
}

pub struct D1Field {
    value: JsValue,
    name: String,
}

impl<'stmt> Field<'stmt, D1Backend> for D1Field {
    fn field_name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn value(&self) -> Option<D1Value> {
        Some(D1Value(self.value.clone()))
    }
}
