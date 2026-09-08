//! General trait impls for sqlite-like backends.

use crate::{D1Backend, D1TypeName};

#[derive(Debug, Copy, Clone)]
pub struct SqliteOnConflictClause;

impl diesel::backend::sql_dialect::on_conflict_clause::SupportsOnConflictClause
    for SqliteOnConflictClause
{
}
impl diesel::backend::sql_dialect::on_conflict_clause::SupportsOnConflictClauseWhere
    for SqliteOnConflictClause
{
}
impl diesel::backend::sql_dialect::on_conflict_clause::PgLikeOnConflictClause
    for SqliteOnConflictClause
{
}

#[derive(Debug, Copy, Clone)]
pub struct SqliteBatchInsert;

#[derive(Debug, Copy, Clone)]
pub struct SqliteReturningClause;

impl diesel::backend::sql_dialect::returning_clause::SupportsReturningClause
    for SqliteReturningClause
{
}

impl diesel::sql_types::TypeMetadata for D1Backend {
    type TypeMetadata = D1TypeName;
    type MetadataLookup = ();
}

impl diesel::backend::SqlDialect for D1Backend {
    type ReturningClause = SqliteReturningClause;

    type OnConflictClause = SqliteOnConflictClause;

    type InsertWithDefaultKeyword =
        diesel::backend::sql_dialect::default_keyword_for_insert::DoesNotSupportDefaultKeyword;
    type BatchInsertSupport = SqliteBatchInsert;
    type ConcatClause = diesel::backend::sql_dialect::concat_clause::ConcatWithPipesClause;
    type DefaultValueClauseForInsert =
        diesel::backend::sql_dialect::default_value_clause::AnsiDefaultValueClause;

    type EmptyFromClauseSyntax =
        diesel::backend::sql_dialect::from_clause_syntax::AnsiSqlFromClauseSyntax;
    type SelectStatementSyntax =
        diesel::backend::sql_dialect::select_statement_syntax::AnsiSqlSelectStatement;

    type ExistsSyntax = diesel::backend::sql_dialect::exists_syntax::AnsiSqlExistsSyntax;
    type ArrayComparison = diesel::backend::sql_dialect::array_comparison::AnsiSqlArrayComparison;
    type AliasSyntax = diesel::backend::sql_dialect::alias_syntax::AsAliasSyntax;

    // From sqlite dialect
    // https://github.com/diesel-rs/diesel/blob/728f9df49e0a739746a758752ac064453a4c79b2/diesel/src/sqlite/backend.rs#L73-L80

    type WindowFrameClauseGroupSupport =
        diesel::backend::sql_dialect::window_frame_clause_group_support::IsoGroupWindowFrameUnit;
    type WindowFrameExclusionSupport =
        diesel::backend::sql_dialect::window_frame_exclusion_support::FrameExclusionSupport;
    type AggregateFunctionExpressions =
              diesel::backend::sql_dialect::aggregate_function_expressions::PostgresLikeAggregateFunctionExpressions;
    type BuiltInWindowFunctionRequireOrder =
        diesel::backend::sql_dialect::built_in_window_function_require_order::NoOrderRequired;
}

impl diesel::backend::DieselReserveSpecialization for D1Backend {}
impl diesel::backend::TrustedBackend for D1Backend {}
