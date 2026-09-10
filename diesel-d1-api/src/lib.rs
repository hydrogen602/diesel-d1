use diesel::connection::TransactionManagerStatus;
use diesel_d1_core::{D1Error, D1TransactionManager};
use reqwest::Url;
use uuid::Uuid;

use crate::utils::NoShowString;

mod client;
mod connection;
mod utils;

const DEFAULT_URL: &str = "https://api.cloudflare.com/client/v4/";

pub struct D1Connection<C> {
    transaction_manager: D1TransactionManager,
    transaction_status: TransactionManagerStatus,

    url: String,
    bearer_token: NoShowString,

    client: C,
}

impl<C> D1Connection<C> {
    /// Constructs a new `D1Connection` with the given base URL, account ID, database ID, and bearer token.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the D1 API. Defaults to `https://api.cloudflare.com/client/v4/`.
    /// * `account_id` - The ID of the account to connect to.
    /// * `database_id` - The ID of the database to connect to.
    /// * `bearer_token` - The bearer token to use for authentication.
    pub fn new(
        base_url: Option<&str>,
        account_id: &str,
        database_id: Uuid,
        bearer_token: NoShowString,
        client: C,
    ) -> Result<Self, D1Error> {
        let base_url = base_url.unwrap_or(DEFAULT_URL);
        let url = base_url.trim_end_matches('/');
        let account_id = urlencoding::encode(account_id);

        // accounts/$ACCOUNT_ID/d1/database/$DATABASE_ID/raw
        let url = format!("{url}/accounts/{account_id}/d1/database/{database_id}/raw",);

        Ok(Self {
            url,
            bearer_token,

            transaction_manager: D1TransactionManager::UnimplementedPanic,
            transaction_status: TransactionManagerStatus::default(),

            client,
        })
    }
}
