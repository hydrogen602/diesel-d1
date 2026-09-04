use diesel::{
    QueryResult,
    connection::{TransactionManagerStatus, ValidTransactionManagerStatus},
};
use diesel_async::{AsyncConnection, TransactionManager};

use crate::utils::D1Error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
/// D1 doesn't have transactions other than batches
///
/// Here are some options for handling it:
pub enum D1TransactionManager {
    #[default]
    /// panic!
    UnimplementedPanic,
    /// return a diesel error
    QueryError,
    /// Ignore the transaction and do nothing.
    Ignore,
}

impl D1TransactionManager {
    pub fn into_status(self) -> TransactionManagerStatus {
        match self {
            D1TransactionManager::UnimplementedPanic => TransactionManagerStatus::InError,
            D1TransactionManager::QueryError => TransactionManagerStatus::InError,
            D1TransactionManager::Ignore => {
                // never in a transaction
                TransactionManagerStatus::Valid(ValidTransactionManagerStatus::default())
            }
        }
    }
}

pub trait D1Connection: AsyncConnection {
    fn transaction_manager(&self) -> D1TransactionManager;
    /// We need some mutable ref to placate diesel.
    ///
    /// This should match what
    /// ```
    /// self.transaction_manager().into_status()
    /// ```
    /// would return.
    fn transaction_manager_status_mut(&mut self) -> &mut TransactionManagerStatus;
}

impl<C: D1Connection> TransactionManager<C> for D1TransactionManager {
    type TransactionStateData = Self;

    async fn begin_transaction(conn: &mut C) -> QueryResult<()> {
        match conn.transaction_manager() {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => {
                Err(D1Error::new("D1 doesn't have transactions other than batches").into())
            }
            D1TransactionManager::Ignore => Ok(()),
        }
    }

    async fn rollback_transaction(conn: &mut C) -> QueryResult<()> {
        match conn.transaction_manager() {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => {
                Err(D1Error::new("D1 doesn't have transactions other than batches").into())
            }
            D1TransactionManager::Ignore => Ok(()),
        }
    }

    async fn commit_transaction(conn: &mut C) -> QueryResult<()> {
        match conn.transaction_manager() {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => {
                Err(D1Error::new("D1 doesn't have transactions other than batches").into())
            }
            D1TransactionManager::Ignore => Ok(()),
        }
    }

    fn transaction_manager_status_mut(conn: &mut C) -> &mut TransactionManagerStatus {
        match conn.transaction_manager() {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => conn.transaction_manager_status_mut(),
            D1TransactionManager::Ignore => conn.transaction_manager_status_mut(),
        }
    }
}
