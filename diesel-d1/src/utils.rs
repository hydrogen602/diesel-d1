use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use diesel::result::DatabaseErrorInformation;
use wasm_bindgen::JsValue;

/// Basically, JS promises are never sendable - they just exist in one thread. While this could be a problem
/// for multi-threaded WASM environments. However, Cloudflare Workers are ALWAYS single-threaded, so we can make
/// every JSFuture sendable by using this wrapper. Useful for stuff that uses `async_trait` (and makes the future not sendable)
pub struct SendableFuture<T>(pub T)
where
    T: Future;

// Safety: WebAssembly will only ever run in a single-threaded context.
unsafe impl<T: Future> Send for SendableFuture<T> {}

// Implement Future for SendableFuture
impl<T> Future for SendableFuture<T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: We are only pinning the inner future.
        unsafe { self.map_unchecked_mut(|s| &mut s.0).poll(cx) }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("D1 error: {message}")]
/// FIXME: this isn't the best error type
pub struct D1Error {
    pub message: String,
}

impl From<worker::Error> for D1Error {
    fn from(error: worker::Error) -> Self {
        D1Error {
            message: error.to_string(),
        }
    }
}

impl From<JsValue> for D1Error {
    fn from(value: JsValue) -> Self {
        let message = value
            .as_string()
            .unwrap_or_else(|| js_sys::JsString::from(value).into());
        D1Error { message }
    }
}

impl From<D1Error> for diesel::result::Error {
    fn from(error: D1Error) -> Self {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(error),
        )
    }
}

impl DatabaseErrorInformation for D1Error {
    fn message(&self) -> &str {
        &self.message
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        None
    }

    fn column_name(&self) -> Option<&str> {
        None
    }

    fn constraint_name(&self) -> Option<&str> {
        None
    }

    fn statement_position(&self) -> Option<i32> {
        None
    }
}
