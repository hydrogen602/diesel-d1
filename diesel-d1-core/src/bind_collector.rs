use diesel::{
    query_builder::BindCollector,
    serialize::{IsNull, Output},
    sql_types::HasSqlType,
};

use crate::{D1TypeName, backend::D1Backend};

#[derive(Debug)]
/// Copied from [worker::D1Type] but added owned variants of ref types.
pub enum D1ValueSendable<'a> {
    Null,
    Real(f64),
    // I believe JS always casts to float. Documentation states it can accept up to 53 bits of signed precision
    // so I went with i32 here. https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures#number_type
    // D1 does not support `BigInt`
    Integer(i32),
    Text(&'a str),
    TextOwned(Box<str>),
    Boolean(bool),
    Blob(&'a [u8]),
    BlobOwned(Box<[u8]>),
}

#[cfg(feature = "worker")]
mod worker_impls {
    use super::*;
    use js_sys::Uint8Array;
    use wasm_bindgen::JsValue;
    use worker::{D1Argument, D1Type};

    impl D1Argument for D1ValueSendable<'_> {
        fn js_value(&self) -> impl AsRef<JsValue> {
            match *self {
                D1ValueSendable::Null => JsValue::null(),
                D1ValueSendable::Real(value) => JsValue::from(value),
                D1ValueSendable::Integer(value) => JsValue::from(value),
                D1ValueSendable::Text(value) => JsValue::from(value),
                D1ValueSendable::TextOwned(ref value) => JsValue::from(value.as_ref()),
                D1ValueSendable::Boolean(value) => JsValue::from(value),
                D1ValueSendable::Blob(value) => JsValue::from(Uint8Array::from(value)),
                D1ValueSendable::BlobOwned(ref value) => {
                    JsValue::from(Uint8Array::from(value.as_ref()))
                }
            }
        }
    }

    impl D1ValueSendable<'_> {
        pub fn as_ref(&self) -> D1Type<'_> {
            match *self {
                D1ValueSendable::Null => D1Type::Null,
                D1ValueSendable::Real(value) => D1Type::Real(value),
                D1ValueSendable::Integer(value) => D1Type::Integer(value),
                D1ValueSendable::Text(value) => D1Type::Text(value),
                D1ValueSendable::TextOwned(ref value) => D1Type::Text(value.as_ref()),
                D1ValueSendable::Boolean(value) => D1Type::Boolean(value),
                D1ValueSendable::Blob(value) => D1Type::Blob(value),
                D1ValueSendable::BlobOwned(ref value) => D1Type::Blob(value.as_ref()),
            }
        }
    }
}

impl From<f64> for D1ValueSendable<'_> {
    fn from(value: f64) -> Self {
        D1ValueSendable::Real(value)
    }
}
impl From<i32> for D1ValueSendable<'_> {
    fn from(value: i32) -> Self {
        D1ValueSendable::Integer(value)
    }
}
impl From<bool> for D1ValueSendable<'_> {
    fn from(value: bool) -> Self {
        D1ValueSendable::Boolean(value)
    }
}
impl<'a> From<&'a str> for D1ValueSendable<'a> {
    fn from(value: &'a str) -> Self {
        D1ValueSendable::Text(value)
    }
}
impl From<Box<str>> for D1ValueSendable<'_> {
    fn from(value: Box<str>) -> Self {
        D1ValueSendable::TextOwned(value)
    }
}
impl From<String> for D1ValueSendable<'_> {
    fn from(value: String) -> Self {
        D1ValueSendable::TextOwned(value.into_boxed_str())
    }
}
impl<'a> From<&'a String> for D1ValueSendable<'a> {
    fn from(value: &'a String) -> Self {
        D1ValueSendable::Text(value.as_str())
    }
}
impl<'a> From<&'a [u8]> for D1ValueSendable<'a> {
    fn from(value: &'a [u8]) -> Self {
        D1ValueSendable::Blob(value)
    }
}
impl From<Box<[u8]>> for D1ValueSendable<'_> {
    fn from(value: Box<[u8]>) -> Self {
        D1ValueSendable::BlobOwned(value)
    }
}

#[derive(Default)]
pub struct D1BindCollector<'bind> {
    pub binds: Vec<(D1ValueSendable<'bind>, D1TypeName)>,
}

impl<'bind> BindCollector<'bind, D1Backend> for D1BindCollector<'bind> {
    type Buffer = D1ValueSendable<'bind>;

    fn push_bound_value<T, U>(
        &mut self,
        bind: &'bind U,
        metadata_lookup: &mut <D1Backend as diesel::sql_types::TypeMetadata>::MetadataLookup,
    ) -> diesel::QueryResult<()>
    where
        D1Backend: diesel::backend::Backend + diesel::sql_types::HasSqlType<T>,
        U: diesel::serialize::ToSql<T, D1Backend> + ?Sized + 'bind,
    {
        let metadata = <D1Backend as HasSqlType<T>>::metadata(metadata_lookup);

        let value = D1ValueSendable::Null; // start out with null
        let mut to_sql_output = Output::new(value, metadata_lookup);
        let is_null = bind
            .to_sql(&mut to_sql_output)
            .map_err(diesel::result::Error::SerializationError)?;

        let bind = to_sql_output.into_inner();
        self.binds.push((
            match is_null {
                IsNull::No => bind,
                IsNull::Yes => D1ValueSendable::Null,
            },
            metadata,
        ));
        Ok(())
    }
}
