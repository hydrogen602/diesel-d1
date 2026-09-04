pub mod traits;
mod transaction_manager;
mod utils;
pub mod value;

pub use transaction_manager::D1Connection;
pub use transaction_manager::D1TransactionManager;
pub use utils::{D1Error, Missing, Present, Required, SendableFuture};

pub mod prelude {
    pub use super::{D1Error, D1TypeName, SendableFuture};
}

/// Determines how a bind parameter is given to SQLite
///
/// Diesel deals with bind parameters after serialization as opaque blobs of
/// bytes. However, SQLite instead has several functions where it expects the
/// relevant C types.
///
/// The variants of this struct determine what bytes are expected from
/// `ToSql` impls.
///
/// Note: Based on [worker::D1Type]
/// - This should be called D1Type but workers already uses that name.
#[allow(missing_debug_implementations)]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum D1TypeName {
    Null,
    Real,
    Integer,
    Text,
    Boolean,
    Blob,
}
