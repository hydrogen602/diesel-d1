use backend::D1Backend;
use bind_collector::D1BindCollector;
use diesel::{
    ConnectionResult, QueryResult,
    connection::{CacheSize, ConnectionSealed, Instrumentation, TransactionManagerStatus},
    query_builder::{AsQuery, QueryFragment, QueryId},
};
use diesel_async::{AsyncConnection, AsyncConnectionCore, SimpleAsyncConnection};
use futures_util::{
    FutureExt, StreamExt,
    future::BoxFuture,
    stream::{self, BoxStream},
};
use js_sys::{Array, Function, Promise, Reflect};
use query_builder::D1QueryBuilder;
use row::D1Row;
use transaction_manager::D1TransactionManager;
use utils::{D1Error, SendableFuture};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker::{D1Database, D1DatabaseSession, D1PreparedStatement, Env};

use crate::bind_collector::D1TypeOwnable;

pub mod backend;
mod bind_collector;
mod builder;
mod query_builder;
mod row;
mod transaction_manager;
mod types;
mod utils;
mod value;
pub use builder::D1ConnectionBuilder;

pub struct D1Connection {
    transaction_manager: D1TransactionManager,
    transaction_status: TransactionManagerStatus,
    binding: D1Database,
    /// If None, no session is used.
    session: Option<D1DatabaseSession>,
}

// impl AsRef<D1Database> for D1Connection {
//     fn as_ref(&self) -> &D1Database {
//         &self.binding
//     }
// }

// impl AsMut<D1Database> for D1Connection {
//     fn as_mut(&mut self) -> &mut D1Database {
//         &mut self.binding
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
/// Supported constraints are `"first-primary"` and `"first-unconstrained"`.
/// Any other value is treated as a bookmark.
/// https://developers.cloudflare.com/d1/worker-api/d1-database/#withsession
pub enum SessionOptions {
    #[default]
    /// This provides the best guarantees.
    FirstPrimary,
    FirstUnconstrained,
    Bookmark(String),
    /// Don't use a session at all.
    DoNotUseSession,
}

impl D1Connection {
    pub(crate) fn new(
        env: &Env,
        name: &str,
        session_constraint: SessionOptions,
        transaction_manager: D1TransactionManager,
    ) -> worker::Result<Self> {
        let binding: D1Database = env.d1(name)?;

        let session = match session_constraint {
            SessionOptions::FirstPrimary => {
                Some(binding.with_session_constraint(worker::D1SessionConstraint::FirstPrimary)?)
            }
            SessionOptions::FirstUnconstrained => Some(
                binding.with_session_constraint(worker::D1SessionConstraint::FirstUnconstrained)?,
            ),
            // The case of passing in None here is equivalent to "first-unconstrained",
            // so we don't need to handle that here as we already provided that option above.
            SessionOptions::Bookmark(bookmark) => Some(binding.with_session(Some(&bookmark))?),
            SessionOptions::DoNotUseSession => None,
        };

        // use sessions
        Ok(D1Connection {
            transaction_manager,
            transaction_status: transaction_manager.into_status(),
            binding,
            session,
        })
    }

    pub(crate) fn db_binding(&self) -> &D1Database {
        &self.binding
    }

    fn prepare_statement_sql<'query, T>(&self, source: T) -> QueryResult<D1PreparedStatement>
    where
        T: QueryFragment<D1Backend> + QueryId + 'query,
    {
        let mut query_builder = D1QueryBuilder::default();
        source.to_sql(&mut query_builder, &D1Backend)?;
        let sql = query_builder.sql;

        // if we use a session, use it. Otherwise, use the database binding.
        let result = match &self.session {
            Some(session) => session.prepare(sql),
            None => self.binding.prepare(sql),
        };

        let binds = construct_bind_data(&source)?;

        result.bind_refs(binds.iter()).map_err(|err| {
            D1Error {
                message: err.to_string(),
            }
            .into()
        })
    }
}

// SAFETY: this is safe under WASM and workers because there's no threads and therefore no race conditions (at least memory ones)
unsafe impl Send for D1Connection {}
unsafe impl Sync for D1Connection {}

impl SimpleAsyncConnection for D1Connection {
    /// WARNING:
    /// This is not a d1 batch, as that requires a Vec of prepared statements,
    /// but we only get a &str.
    ///
    /// A possible solution would be to parse the sql and prepare the statements
    ///
    /// WARNING: This also can't use the session API, so if this batch_execute does a write,
    /// subsequent reads are not guaranteed to see the changes.
    ///
    /// FIXME: both of the warnings could be removed if we can split the query
    /// into individual statements and prepare them individually.
    async fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        match SendableFuture(self.db_binding().exec(query)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(D1Error::from(e).into()),
        }
    }
}

impl AsyncConnection for D1Connection {
    type TransactionManager = D1TransactionManager;

    async fn establish(_unused: &str) -> ConnectionResult<Self> {
        unimplemented!("A D1 connection is made from bindings")
    }

    fn transaction_state(&mut self) -> &mut D1TransactionManager {
        &mut self.transaction_manager
    }

    #[doc(hidden)]
    fn instrumentation(&mut self) -> &mut dyn Instrumentation {
        todo!()
    }

    #[doc = " Set a specific [`Instrumentation`] implementation for this connection"]
    fn set_instrumentation(&mut self, _instrumentation: impl Instrumentation) {
        todo!()
    }

    fn set_prepared_statement_cache_size(&mut self, _size: CacheSize) {
        todo!()
    }
}

impl AsyncConnectionCore for D1Connection {
    type Backend = D1Backend;

    #[doc = " The future returned by `AsyncConnection::execute`"]
    type ExecuteFuture<'conn, 'query> = BoxFuture<'conn, QueryResult<usize>>;

    #[doc = " The future returned by `AsyncConnection::load`"]
    type LoadFuture<'conn, 'query> = BoxFuture<'conn, QueryResult<Self::Stream<'conn, 'query>>>;

    #[doc = " The inner stream returned by `AsyncConnection::load`"]
    type Stream<'conn, 'query> = BoxStream<'conn, QueryResult<Self::Row<'conn, 'query>>>;

    #[doc = " The row type used by the stream returned by `AsyncConnection::load`"]
    type Row<'conn, 'query> = D1Row;

    fn load<'conn, 'query, T>(&'conn mut self, source: T) -> Self::LoadFuture<'conn, 'query>
    where
        T: AsQuery + 'query,
        T::Query: QueryFragment<Self::Backend> + QueryId + 'query,
    {
        let source = source.as_query();
        match self.prepare_statement_sql(source) {
            Ok(result) => SendableFuture(async move {
                let rows = match raw_with_column_names(result).await {
                    Ok(rows) => rows,
                    Err(err) => {
                        return Err(D1Error::from(err).into());
                    }
                };

                // we could maybe inject our own limit and offset to fetch the results in multiple pieces.
                Ok(stream::iter(rows).boxed())
            })
            .boxed(),
            Err(err) => SendableFuture(async move { Err(err) }).boxed(),
        }
    }

    #[doc(hidden)]
    fn execute_returning_count<'conn, 'query, T>(
        &'conn mut self,
        source: T,
    ) -> Self::ExecuteFuture<'conn, 'query>
    where
        T: QueryFragment<Self::Backend> + QueryId + 'query,
    {
        match self.prepare_statement_sql(source) {
            Ok(result) => SendableFuture(async move {
                let result = match result.run().await {
                    Ok(res) => res,
                    Err(err) => {
                        return Err(D1Error::from(err).into());
                    }
                };

                if let Some(error_str) = result.error() {
                    return Err(D1Error { message: error_str }.into());
                }

                // if it's successful, meta exists with a `changes` key that is a number
                let meta = result
                    .meta()
                    .map_err(D1Error::from)?
                    .ok_or_else(|| D1Error {
                        message: "D1 didn't return meta property".to_string(),
                    })?;
                let value = meta.changes.ok_or_else(|| D1Error {
                    message: "D1 didn't return change property".to_string(),
                })?;

                Ok(value)
            })
            .boxed(),
            Err(err) => SendableFuture(async move { Err(err) }).boxed(),
        }
    }
}

impl ConnectionSealed for D1Connection {}

fn construct_bind_data<T>(query: &T) -> Result<Vec<D1TypeOwnable<'_>>, diesel::result::Error>
where
    T: QueryFragment<D1Backend>,
{
    let mut bind_collector = D1BindCollector::default();

    query.collect_binds(&mut bind_collector, &mut (), &D1Backend)?;

    let array = bind_collector
        .binds
        .into_iter()
        .map(|(bind, _)| bind)
        .collect::<Vec<_>>();
    Ok(array)
}

/// D1 rust bindings don't support this call yet, but the underlying JS API does.
///
/// ```js
/// stmt.raw({columnNames: true})
/// ```
///
/// https://developers.cloudflare.com/d1/worker-api/prepared-statements/#raw
async fn raw_with_column_names(
    stmt: D1PreparedStatement,
) -> worker::Result<Vec<QueryResult<D1Row>>> {
    let this = stmt.inner();
    let raw_fn = Reflect::get(this, &JsValue::from_str("raw"))?
        .dyn_into::<Function>()
        .map_err(worker::Error::from)?;
    let opts = js_sys::Object::new(); // FIXME: cache this object?
    Reflect::set(&opts, &JsValue::from_str("columnNames"), &JsValue::TRUE)?;
    let promise = raw_fn
        .call1(this, &opts)?
        .dyn_into::<Promise>()
        .map_err(worker::Error::from)?;
    let result = JsFuture::from(promise).await?;
    let array = result.dyn_into::<Array>().map_err(worker::Error::from)?;

    let Some(column_names) = array.shift_checked() else {
        return Err(worker::Error::RustError(
            "D1 didn't return column names".to_string(),
        ));
    };
    let column_names: Vec<String> = column_names
        .dyn_into::<Array>()
        .map_err(worker::Error::from)?
        .iter()
        .map(|key| {
            key.as_string().ok_or_else(|| {
                worker::Error::RustError("D1 column name was not a string".to_string())
            })
        })
        .collect::<worker::Result<_>>()?;

    // we shifted the array so we only have data rows left
    Ok(array
        .into_iter()
        .map(|value| -> QueryResult<D1Row> {
            let values = value.dyn_into::<Array>().map_err(D1Error::from)?;
            if values.length() as usize != column_names.len() {
                return QueryResult::Err(
                    D1Error {
                        message: format!(
                            "D1 row has {} values but {} column names",
                            values.length(),
                            column_names.len()
                        ),
                    }
                    .into(),
                );
            }
            let fields = column_names.iter().cloned().zip(values).collect::<Vec<_>>();

            Ok(D1Row::from_named_values(fields))
        })
        .collect())
}
