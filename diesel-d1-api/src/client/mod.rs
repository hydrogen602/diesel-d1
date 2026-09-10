use crate::{
    client::models::{RawQueryRequest, RawQueryResponse},
    utils::NoShowString,
};

pub mod models;

/// A trait for making asynchronous HTTP requests.
///
/// Implement this trait if you want to bring your own HTTP client.
pub trait AsyncHttpClient {
    type Error;
    async fn post(
        &self,
        url: &str,
        bearer_token: &NoShowString,
        body: RawQueryRequest,
    ) -> Result<RawQueryResponse, Self::Error>;
}

#[cfg(feature = "reqwest")]
impl AsyncHttpClient for reqwest::Client {
    type Error = reqwest::Error;
    async fn post(
        &self,
        url: &str,
        bearer_token: &NoShowString,
        body: RawQueryRequest,
    ) -> Result<RawQueryResponse, Self::Error> {
        self.post(url)
            .bearer_auth(bearer_token.get_str())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<RawQueryResponse>()
            .await
    }
}
