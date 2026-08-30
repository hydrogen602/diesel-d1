use diesel::{QueryResult, connection::TransactionManagerStatus};
use diesel_async::TransactionManager;

use crate::D1Connection;

#[derive(Default)]
/// D1 doesn't have transactions other than batches
pub struct D1TransactionManager;

impl TransactionManager<D1Connection> for D1TransactionManager {
    type TransactionStateData = Self;

    async fn begin_transaction(_conn: &mut D1Connection) -> QueryResult<()> {
        unimplemented!("D1 doesn't have transactions other than batches")
    }

    async fn rollback_transaction(_conn: &mut D1Connection) -> QueryResult<()> {
        unimplemented!("D1 doesn't have transactions other than batches")
    }

    async fn commit_transaction(_conn: &mut D1Connection) -> QueryResult<()> {
        unimplemented!("D1 doesn't have transactions other than batches")
    }

    fn transaction_manager_status_mut(_conn: &mut D1Connection) -> &mut TransactionManagerStatus {
        unimplemented!("D1 doesn't have transactions other than batches")
    }
}
