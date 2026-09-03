use diesel::backend::sql_dialect::default_keyword_for_insert::DoesNotSupportDefaultKeyword;
use diesel::expression::{AppearsOnTable, Expression};
use diesel::insertable::{
    CanInsertInSingleQuery, ColumnInsertValue, DefaultableColumnInsertValue, InsertValues,
};
use diesel::query_builder::{AstPass, BatchInsert, NoFromClause, QueryFragment, ValuesClause};
use diesel::query_source::Column;
use diesel::result::QueryResult;

use crate::backend::{D1Backend, SqliteBatchInsert};

impl<Col, Expr> InsertValues<D1Backend, Col::Table>
    for DefaultableColumnInsertValue<ColumnInsertValue<Col, Expr>>
where
    Col: Column,
    Expr: Expression<SqlType = Col::SqlType> + AppearsOnTable<NoFromClause>,
    Self: QueryFragment<D1Backend>,
{
    fn column_names(&self, mut out: AstPass<'_, '_, D1Backend>) -> QueryResult<()> {
        if let Self::Expression(..) = *self {
            out.push_identifier(Col::NAME)?;
        }
        Ok(())
    }
}

impl<Col, Expr> QueryFragment<D1Backend, DoesNotSupportDefaultKeyword>
    for DefaultableColumnInsertValue<ColumnInsertValue<Col, Expr>>
where
    Expr: QueryFragment<D1Backend>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, D1Backend>) -> QueryResult<()> {
        if let Self::Expression(ref inner) = *self {
            inner.walk_ast(out.reborrow())?;
        }
        Ok(())
    }
}

impl<V, Tab, QId, const STATIC_QUERY_ID: bool> CanInsertInSingleQuery<D1Backend>
    for BatchInsert<Vec<ValuesClause<V, Tab>>, Tab, QId, STATIC_QUERY_ID>
where
    V: CanInsertInSingleQuery<D1Backend>,
{
    fn rows_to_insert(&self) -> Option<usize> {
        Some(self.values.len())
    }
}

impl<V, Tab, QId, const STATIC_QUERY_ID: bool> QueryFragment<D1Backend, SqliteBatchInsert>
    for BatchInsert<Vec<ValuesClause<V, Tab>>, Tab, QId, STATIC_QUERY_ID>
where
    ValuesClause<V, Tab>: QueryFragment<D1Backend>,
    V: QueryFragment<D1Backend>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, D1Backend>) -> QueryResult<()> {
        if !STATIC_QUERY_ID {
            out.unsafe_to_cache_prepared();
        }

        let mut values = self.values.iter();
        if let Some(value) = values.next() {
            value.walk_ast(out.reborrow())?;
        }
        for value in values {
            out.push_sql(", (");
            value.values.walk_ast(out.reborrow())?;
            out.push_sql(")");
        }
        Ok(())
    }
}
