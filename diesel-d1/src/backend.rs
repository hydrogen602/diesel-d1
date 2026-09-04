use diesel::backend::Backend;
use diesel_d1_core::sqlite_dialect_impl;

use crate::{bind_collector::D1BindCollector, query_builder::D1QueryBuilder, value::D1Value};

/// The SQLite backend
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default)]
pub struct D1Backend;

impl Backend for D1Backend {
    type QueryBuilder = D1QueryBuilder;
    type RawValue<'a> = D1Value;
    type BindCollector<'a> = D1BindCollector<'a>;
}

sqlite_dialect_impl!(D1Backend);
