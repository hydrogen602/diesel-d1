use diesel::{
    backend::{
        Backend, DieselReserveSpecialization, SqlDialect, TrustedBackend,
        sql_dialect::{self, returning_clause::DoesNotSupportReturningClause},
    },
    sql_types::TypeMetadata,
};

use crate::{bind_collector::D1BindCollector, query_builder::D1QueryBuilder, value::D1Value};

/// The SQLite backend
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default)]
pub struct D1Backend;

/// Determines how a bind parameter is given to SQLite
///
/// Diesel deals with bind parameters after serialization as opaque blobs of
/// bytes. However, SQLite instead has several functions where it expects the
/// relevant C types.
///
/// The variants of this struct determine what bytes are expected from
/// `ToSql` impls.
#[allow(missing_debug_implementations)]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
// sqlite types
pub enum D1Type {
    Binary,
    Text,
    Double,
    Integer,
}

impl Backend for D1Backend {
    type QueryBuilder = D1QueryBuilder;
    type RawValue<'a> = D1Value;
    type BindCollector<'a> = D1BindCollector;
}

impl TypeMetadata for D1Backend {
    type TypeMetadata = D1Type;
    type MetadataLookup = ();
}

impl SqlDialect for D1Backend {
    // this is actually not true, but i would need to properly implement the ast for this, since the sqlite one is not exported
    type ReturningClause = DoesNotSupportReturningClause;

    type OnConflictClause = SqliteOnConflictClause;

    type InsertWithDefaultKeyword =
        sql_dialect::default_keyword_for_insert::DoesNotSupportDefaultKeyword;
    type BatchInsertSupport = SqliteBatchInsert;
    type ConcatClause = sql_dialect::concat_clause::ConcatWithPipesClause;
    type DefaultValueClauseForInsert = sql_dialect::default_value_clause::AnsiDefaultValueClause;

    type EmptyFromClauseSyntax = sql_dialect::from_clause_syntax::AnsiSqlFromClauseSyntax;
    type SelectStatementSyntax = sql_dialect::select_statement_syntax::AnsiSqlSelectStatement;

    type ExistsSyntax = sql_dialect::exists_syntax::AnsiSqlExistsSyntax;
    type ArrayComparison = sql_dialect::array_comparison::AnsiSqlArrayComparison;
    type AliasSyntax = sql_dialect::alias_syntax::AsAliasSyntax;

    // From sqlite dialect
    // https://github.com/diesel-rs/diesel/blob/728f9df49e0a739746a758752ac064453a4c79b2/diesel/src/sqlite/backend.rs#L73-L80

    type WindowFrameClauseGroupSupport =
        sql_dialect::window_frame_clause_group_support::IsoGroupWindowFrameUnit;
    type WindowFrameExclusionSupport =
        sql_dialect::window_frame_exclusion_support::FrameExclusionSupport;
    type AggregateFunctionExpressions =
        sql_dialect::aggregate_function_expressions::PostgresLikeAggregateFunctionExpressions;
    type BuiltInWindowFunctionRequireOrder =
        sql_dialect::built_in_window_function_require_order::NoOrderRequired;
}

impl DieselReserveSpecialization for D1Backend {}
impl TrustedBackend for D1Backend {}

#[derive(Debug, Copy, Clone)]
pub struct SqliteOnConflictClause;

impl sql_dialect::on_conflict_clause::SupportsOnConflictClause for SqliteOnConflictClause {}
impl sql_dialect::on_conflict_clause::PgLikeOnConflictClause for SqliteOnConflictClause {}

#[derive(Debug, Copy, Clone)]
pub struct SqliteBatchInsert;

#[derive(Debug, Copy, Clone)]
pub struct SqliteReturningClause;

impl sql_dialect::returning_clause::SupportsReturningClause for SqliteReturningClause {}
