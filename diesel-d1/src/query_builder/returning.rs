use diesel::backend::Backend;
use diesel::query_builder::ReturningClause;
use diesel::query_builder::{AstPass, QueryFragment};
use diesel::result::QueryResult;
use crate::backend::{D1Backend, SqliteReturningClause};

// impl<Expr> QueryFragment<D1Backend, SqliteReturningClause> for ReturningClause<Expr>
// where
//     Expr: QueryFragment<D1Backend>,
// {
//     fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, D1Backend>) -> QueryResult<()> {
//         out.skip_from(true);
//         out.push_sql(" RETURNING ");
//         self.0.walk_ast(out.reborrow())?;
//         Ok(())
//     }
// }
