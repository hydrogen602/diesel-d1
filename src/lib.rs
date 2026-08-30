use async_trait::async_trait;
use backend::D1Backend;
use bind_collector::D1BindCollector;
use binding::{D1Database, D1PreparedStatement, D1Result};
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
use js_sys::{Array, Object, Reflect};
use query_builder::D1QueryBuilder;
use row::D1Row;
use transaction_manager::D1TransactionManager;
use utils::{D1Error, SendableFuture};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use worker::console_error;

pub mod backend;
mod bind_collector;
mod binding;
mod query_builder;
mod row;
mod transaction_manager;
mod types;
mod utils;
mod value;

pub struct D1Connection {
    transaction_queries: Vec<D1PreparedStatement>,
    transaction_manager: D1TransactionManager,
    binding: D1Database,
}

impl D1Connection {
    pub fn new(env: worker::Env, name: &str) -> Self {
        let binding: D1Database = Reflect::get(&env, &name.to_owned().into()).unwrap().into();
        D1Connection {
            transaction_queries: Vec::default(),
            transaction_manager: D1TransactionManager::default(),
            binding,
        }
    }
}

// SAFETY: this is safe under WASM and workers because there's no threads and therefore no race conditions (at least memory ones)
unsafe impl Send for D1Connection {}
unsafe impl Sync for D1Connection {}

impl SimpleAsyncConnection for D1Connection {
    async fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        let statements = [JsValue::from_str(query)].iter().collect::<Array>();

        match SendableFuture(JsFuture::from(self.binding.batch(statements).unwrap())).await {
            Ok(_) => Ok(()),
            // FIXME(lduarte): I don't send a proper error becase I don't have time at the moment
            Err(_) => Err(diesel::result::Error::NotFound),
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

#[async_trait]
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
            let promise = match result.all() {
                Ok(res) => res,
                Err(err) => {
                    console_error!("{:?}", err);
                    panic!("not supposed to happen .all call");
                }
            };

            let result = match SendableFuture(JsFuture::from(promise)).await {
                Ok(res) => res,
                Err(err) => {
                    console_error!("{:?}", err);
                    panic!("not supposed to happen .all promise");
                }
            };

            let result: D1Result = result.into();

            let error = result.error().unwrap();

            if let Some(error_str) = error {
                return Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(D1Error { message: error_str }),
                ));
            }

            let array = result.results().unwrap().unwrap().to_vec();

            if array.is_empty() {
                return Ok(stream::iter(vec![]).boxed());
            }

            let field_keys: Vec<String> = js_sys::Object::keys(&Object::from(array[0].clone()))
                .to_vec()
                .iter()
                .map(|val| val.as_string().unwrap())
                .collect();

            // FIXME: not performant at all, should work well enough
            let rows: Vec<QueryResult<D1Row>> = array
                .iter()
                .map(|val| Ok(D1Row::new(val.clone(), field_keys.clone())))
                .collect();
            let iter = stream::iter(rows).boxed();
            Ok(iter)
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
            let promise = match result.all() {
                Ok(res) => res,
                Err(err) => {
                    console_error!("{:?}", err);
                    panic!("not supposed to happen .all call");
                }
            };

            let result = match SendableFuture(JsFuture::from(promise)).await {
                Ok(res) => res,
                Err(err) => {
                    console_error!("{:?}", err);
                    panic!("not supposed to happen .all promise");
                }
            };

            let result: D1Result = result.into();

            let error = result.error().unwrap();

            if let Some(error_str) = error {
                return Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(D1Error { message: error_str }),
                ));
            }

            // if it's successful, meta exists with a `changes` key that is a number
            let meta = result.meta().unwrap();
            let value = js_sys::Reflect::get(&meta, &"changes".to_owned().into())
                .unwrap()
                .as_f64()
                .unwrap();

            Ok(value as usize)
        })
        .boxed()
    }
}

impl ConnectionSealed for D1Connection {}

fn construct_bind_data<T>(query: &T) -> Result<Array, diesel::result::Error>
where
    T: QueryFragment<D1Backend>,
{
    let mut bind_collector = D1BindCollector::default();

    query.collect_binds(&mut bind_collector, &mut (), &D1Backend)?;

    let array = bind_collector
        .binds
        .iter()
        .map(|(bind, _)| bind)
        .collect::<Array>();
    Ok(array)
}

fn prepare_statement_sql<'conn, 'query, T>(source: T, binding: &D1Database) -> D1PreparedStatement
where
    T: QueryFragment<D1Backend> + QueryId + 'query,
{
    let mut query_builder = D1QueryBuilder::default();
    source.to_sql(&mut query_builder, &D1Backend).unwrap();
    let result = match binding.prepare(&query_builder.sql) {
        Ok(res) => res,
        Err(err) => {
            console_error!("{:?}", err);
            panic!("not supposed to happen d1preparedstatement");
        }
    };

    let binds = construct_bind_data(&source).unwrap();

    match result.bind(binds) {
        Ok(res) => res,
        Err(err) => {
            console_error!("{:?}", err);
            panic!("not supposed to happen bind");
        }
    }
}
