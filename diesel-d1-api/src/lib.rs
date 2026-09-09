use diesel::connection::TransactionManagerStatus;
use diesel_d1_core::{D1Error, D1TransactionManager};
use reqwest::Url;
use uuid::Uuid;

use crate::utils::NoShowString;

mod client;
mod models;
mod utils;

const DEFAULT_URL: &str = "https://api.cloudflare.com/client/v4/";

pub struct D1Connection {
    transaction_manager: D1TransactionManager,
    transaction_status: TransactionManagerStatus,

    url: Url,
    bearer_token: NoShowString,
}

impl D1Connection {
    pub fn new(
        base_url: Option<&str>,
        account_id: &str,
        database_id: Uuid,
        bearer_token: NoShowString,
    ) -> Result<Self, D1Error> {
        let base_url = base_url.unwrap_or(DEFAULT_URL);
        let url = Url::parse(base_url).map_err(|e| D1Error::new(e.to_string()))?;
        // accounts/$ACCOUNT_ID/d1/database/$DATABASE_ID/raw
        let url = url
            .join(&format!(
                "accounts/{}/d1/database/{}/raw",
                urlencoding::encode(account_id),
                database_id
            ))
            .map_err(|e| D1Error::new(e.to_string()))?;

        Ok(Self {
            url,
            bearer_token,

            transaction_manager: D1TransactionManager::UnimplementedPanic,
            transaction_status: TransactionManagerStatus::default(),
        })
    }
}
