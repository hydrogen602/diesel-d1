use diesel::backend::Backend;

use crate::{bind_collector::D1BindCollector, query_builder::D1QueryBuilder, value::D1Value};

/// The SQLite backend
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default)]
pub struct D1Backend;

impl Backend for D1Backend {
    type QueryBuilder = D1QueryBuilder;
    type RawValue<'a> = D1Value<'a>;
    type BindCollector<'a> = D1BindCollector<'a>;
}
