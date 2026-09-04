use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use diesel::result::DatabaseErrorInformation;

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

impl D1Error {
    pub fn new(message: impl ToString) -> Self {
        D1Error {
            message: message.to_string(),
        }
    }
}

#[cfg(feature = "worker")]
impl From<worker::Error> for D1Error {
    fn from(error: worker::Error) -> Self {
        D1Error {
            message: error.to_string(),
        }
    }
}

#[cfg(feature = "worker")]
impl From<wasm_bindgen::JsValue> for D1Error {
    fn from(value: wasm_bindgen::JsValue) -> Self {
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

// builder utils

mod private {
    pub trait Sealed {}
}

pub trait Required<T>: private::Sealed {}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Missing;
impl private::Sealed for Missing {}
impl<T> Required<T> for Missing {}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Present<T>(pub T);
impl<T> private::Sealed for Present<T> {}
impl<T> Required<T> for Present<T> {}

impl<T> Present<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
