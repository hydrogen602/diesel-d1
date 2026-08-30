use diesel::{
    query_builder::BindCollector,
    serialize::{IsNull, Output},
    sql_types::HasSqlType,
};
use wasm_bindgen::JsValue;

use crate::backend::{D1Backend, D1Type};

#[derive(Default)]
pub struct D1BindCollector {
    pub binds: Vec<(JsValue, D1Type)>,
}

impl<'bind> BindCollector<'bind, D1Backend> for D1BindCollector {
    type Buffer = JsValue;

    fn push_bound_value<T, U>(
        &mut self,
        bind: &'bind U,
        metadata_lookup: &mut <D1Backend as diesel::sql_types::TypeMetadata>::MetadataLookup,
    ) -> diesel::QueryResult<()>
    where
        D1Backend: diesel::backend::Backend + diesel::sql_types::HasSqlType<T>,
        U: diesel::serialize::ToSql<T, D1Backend> + ?Sized + 'bind,
    {
        let value = JsValue::null(); // start out with null
        let mut to_sql_output = Output::new(value, metadata_lookup);
        let is_null = bind
            .to_sql(&mut to_sql_output)
            .map_err(diesel::result::Error::SerializationError)?;

        let bind = if matches!(is_null, IsNull::No) {
            to_sql_output.into_inner()
        } else {
            JsValue::null()
        };

        let metadata = <D1Backend as HasSqlType<T>>::metadata(metadata_lookup);
        self.binds.push((bind, metadata));
        Ok(())
    }
}
