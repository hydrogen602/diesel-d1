use diesel::{
    ConnectionResult, QueryResult,
    connection::{CacheSize, ConnectionSealed, Instrumentation, TransactionManagerStatus},
    query_builder::{AsQuery, QueryFragment, QueryId},
};
use diesel_async::{AsyncConnection, AsyncConnectionCore, SimpleAsyncConnection};
use diesel_d1_core::{
    D1Backend, D1Error, D1TransactionManager, D1ValueSendable, bind_collector::D1BindCollector,
    query_builder::D1QueryBuilder, row::D1Row,
};
use futures_util::{
    future::BoxFuture,
    stream::{self, BoxStream},
};

use crate::{
    D1Connection,
    client::{
        AsyncHttpClient,
        models::{D1SingleQuery, RawQueryRequest},
    },
};

// SAFETY: this is safe under WASM and workers because there's no threads and therefore no race conditions (at least memory ones)
unsafe impl<C: Send> Send for D1Connection<C> {}
unsafe impl<C: Sync> Sync for D1Connection<C> {}

impl<C: AsyncHttpClient> D1Connection<C> {
    async fn run_query(&self, query: StatementWithBinds<'_>) {
        let url = &self.url;
        let bearer_token = &self.bearer_token;
        let body = RawQueryRequest::Single(D1SingleQuery {
            sql: query.sql,
            params: query.binds.iter().map(|bind| bind.to_string()).collect(),
        });

        todo!()
    }
}

impl<C: AsyncHttpClient> SimpleAsyncConnection for D1Connection<C> {
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
        // match SendableFuture(self.db_binding().exec(query)).await {
        //     Ok(_) => Ok(()),
        //     Err(e) => Err(D1Error::from(e).into()),
        // }
        self.client.post
    }
}

impl diesel_d1_core::D1Connection for D1Connection {
    fn transaction_manager(&self) -> D1TransactionManager {
        self.transaction_manager
    }

    fn transaction_manager_status_mut(&mut self) -> &mut TransactionManagerStatus {
        &mut self.transaction_status
    }
}

impl AsyncConnection for D1Connection {
    type TransactionManager = D1TransactionManager;

    async fn establish(_unused: &str) -> ConnectionResult<Self> {
        unimplemented!("Use the new function")
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
    type Row<'conn, 'query> = D1Row<'static>;

    fn load<'conn, 'query, T>(&'conn mut self, source: T) -> Self::LoadFuture<'conn, 'query>
    where
        T: AsQuery + 'query,
        T::Query: QueryFragment<Self::Backend> + QueryId + 'query,
    {
        let source = source.as_query();
        match prepare_statement_sql(self, source) {
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

fn construct_bind_data<T>(query: &T) -> Result<Vec<D1ValueSendable<'_>>, diesel::result::Error>
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
async fn raw_with_column_names<'a>(
    stmt: StatementWithBinds<'a>,
) -> Result<Vec<QueryResult<D1Row<'static>>>, D1Error> {
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

    let column_names: Rc<[String]> = column_names.into();

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
            let values: Result<Box<[D1Value]>, NotConvertibleToD1ValueError> =
                values.into_iter().map(D1Value::try_from).try_collect();

            let values = values.map_err(D1Error::new)?;

            Ok(D1Row::new(column_names.clone(), values))
        })
        .collect())
}

pub struct StatementWithBinds<'a> {
    pub(crate) sql: String,
    pub(crate) binds: Vec<D1ValueSendable<'a>>,
}

impl<'a> StatementWithBinds<'a> {
    pub fn new(sql: String, binds: Vec<D1ValueSendable<'a>>) -> Self {
        Self { sql, binds }
    }
}

fn prepare_statement_sql<'query, T>(
    conn: &D1Connection,
    source: T,
) -> QueryResult<StatementWithBinds<'query>>
where
    T: QueryFragment<D1Backend> + QueryId + 'query,
{
    let mut query_builder = D1QueryBuilder::default();
    source.to_sql(&mut query_builder, &D1Backend)?;
    let sql = query_builder.finish();

    let binds = construct_bind_data(&source)?;

    Ok(StatementWithBinds { sql, binds })
}
