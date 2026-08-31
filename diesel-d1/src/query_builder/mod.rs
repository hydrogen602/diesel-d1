//! The SQLite query builder

use super::backend::D1Backend;
use diesel::query_builder::QueryBuilder;
use diesel::result::QueryResult;

mod limit_offset;
// mod query_fragment_impls;
mod returning;

/// Constructs SQL queries for use with the SQLite backend
#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct D1QueryBuilder {
    pub(crate) sql: String,
}

impl D1QueryBuilder {
    /// Construct a new query builder with an empty query
    pub fn new() -> Self {
        D1QueryBuilder::default()
    }
}

impl QueryBuilder<D1Backend> for D1QueryBuilder {
    fn push_sql(&mut self, sql: &str) {
        self.sql.push_str(sql);
    }

    fn push_identifier(&mut self, identifier: &str) -> QueryResult<()> {
        self.push_sql("`");
        self.push_sql(&identifier.replace('`', "``"));
        self.push_sql("`");
        Ok(())
    }

    fn push_bind_param(&mut self) {
        self.push_sql("?");
    }

    fn finish(self) -> String {
        self.sql
    }
}
