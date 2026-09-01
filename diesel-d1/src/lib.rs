use backend::D1Backend;
use bind_collector::D1BindCollector;
use diesel::{
    ConnectionResult, QueryResult,
    connection::{CacheSize, ConnectionSealed, Instrumentation},
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
use worker::{D1Database, D1PreparedStatement, Env, console_error};

use crate::bind_collector::D1TypeOwnable;

pub mod backend;
mod bind_collector;
mod query_builder;
mod row;
mod transaction_manager;
mod types;
mod utils;
mod value;

pub struct D1Connection {
    transaction_manager: D1TransactionManager,
    binding: D1Database,
}

impl D1Connection {
    pub fn new(env: &Env, name: &str) -> worker::Result<Self> {
        let binding: D1Database = env.d1(name)?;
        // use sessions
        Ok(D1Connection {
            transaction_manager: D1TransactionManager,
            binding,
        })
    }
}

// SAFETY: this is safe under WASM and workers because there's no threads and therefore no race conditions (at least memory ones)
unsafe impl Send for D1Connection {}
unsafe impl Sync for D1Connection {}

impl SimpleAsyncConnection for D1Connection {
    /// FIXME: WARNING:
    /// This is not a d1 batch, as that requires a Vec of prepared statements,
    /// but we only get a &str.
    ///
    /// A possible solution would be to parse the sql and prepare the statements
    async fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        match SendableFuture(self.binding.exec(query)).await {
            Ok(_) => Ok(()),
            // FIXME(lduarte): I don't send a proper error becase I don't have time at the moment
            Err(e) => Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(D1Error::from(e)),
            )),
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
        let result = prepare_statement_sql(source, &self.binding);

        SendableFuture(async move {
            let rows = match raw_with_column_names(result).await {
                Ok(rows) => rows,
                Err(err) => {
                    todo!("Error handling: {:?}", err);
                }
            };

            // we could maybe inject our own limit and offset to fetch the results in multiple pieces.
            Ok(stream::iter(rows).boxed())
        })
        .boxed()
    }

    #[doc(hidden)]
    fn execute_returning_count<'conn, 'query, T>(
        &'conn mut self,
        source: T,
    ) -> Self::ExecuteFuture<'conn, 'query>
    where
        T: QueryFragment<Self::Backend> + QueryId + 'query,
    {
        let result = prepare_statement_sql(source, &self.binding);
        SendableFuture(async move {
            let result = match result.run().await {
                Ok(res) => res,
                Err(err) => {
                    todo!("Error handling: {:?}", err);
                }
            };

            if let Some(error_str) = result.error() {
                return Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(D1Error { message: error_str }),
                ));
            }

            // if it's successful, meta exists with a `changes` key that is a number
            let meta = result.meta().unwrap().unwrap();
            let value = meta.changes.unwrap();

            Ok(value)
        })
        .boxed()
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

fn prepare_statement_sql<'conn, 'query, T>(source: T, binding: &D1Database) -> D1PreparedStatement
where
    T: QueryFragment<D1Backend> + QueryId + 'query,
{
    // let mut query_builder = D1QueryBuilder::default();
    // source.to_sql(&mut query_builder, &D1Backend).unwrap();
    // let result = match binding.prepare(&query_builder.sql) {
    //     Ok(res) => res,
    //     Err(err) => {
    //         console_error!("{:?}", err);
    //         panic!("not supposed to happen d1preparedstatement");
    //     }
    // };

    let mut query_builder = D1QueryBuilder::default();
    source.to_sql(&mut query_builder, &D1Backend).unwrap();
    let sql = query_builder.sql;

    let result = binding.prepare(sql);

    let binds = construct_bind_data(&source).unwrap();

    match result.bind_refs(binds.iter()) {
        Ok(res) => res,
        Err(err) => {
            console_error!("{:?}", err);
            panic!("not supposed to happen bind");
        }
    }
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
            let fields = column_names
                .iter()
                .cloned()
                .zip(values.into_iter())
                .collect::<Vec<_>>();

            Ok(D1Row::from_named_values(fields))
        })
        .collect())
}
