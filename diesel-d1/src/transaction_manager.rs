use diesel::{
    QueryResult,
    connection::{TransactionManagerStatus, ValidTransactionManagerStatus},
};
use diesel_async::TransactionManager;

use crate::{D1Connection, utils::D1Error};

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

impl TransactionManager<D1Connection> for D1TransactionManager {
    type TransactionStateData = Self;

    async fn begin_transaction(conn: &mut D1Connection) -> QueryResult<()> {
        match conn.transaction_manager {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => {
                Err(D1Error::new("D1 doesn't have transactions other than batches").into())
            }
            D1TransactionManager::Ignore => Ok(()),
        }
    }

    async fn rollback_transaction(conn: &mut D1Connection) -> QueryResult<()> {
        match conn.transaction_manager {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => {
                Err(D1Error::new("D1 doesn't have transactions other than batches").into())
            }
            D1TransactionManager::Ignore => Ok(()),
        }
    }

    async fn commit_transaction(conn: &mut D1Connection) -> QueryResult<()> {
        match conn.transaction_manager {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => {
                Err(D1Error::new("D1 doesn't have transactions other than batches").into())
            }
            D1TransactionManager::Ignore => Ok(()),
        }
    }

    fn transaction_manager_status_mut(conn: &mut D1Connection) -> &mut TransactionManagerStatus {
        match &mut conn.transaction_manager {
            D1TransactionManager::UnimplementedPanic => {
                unimplemented!("D1 doesn't have transactions other than batches")
            }
            D1TransactionManager::QueryError => &mut conn.transaction_status,
            D1TransactionManager::Ignore => &mut conn.transaction_status,
        }
    }
}
