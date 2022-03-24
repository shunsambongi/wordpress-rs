use std::borrow::Cow;

use async_trait::async_trait;
use http::{Method, Request};
use serde::de::DeserializeOwned;

use crate::{client::Client, query::Query, ApiError};

/// A trait for providing the necessary information for a single REST API
/// endpoint.
pub trait Endpoint {
    /// HTTP method for the endpoint.
    fn method(&self) -> Method;

    /// Route for the endpoint.
    fn route(&self) -> Cow<'static, str>;
}

#[async_trait]
impl<E, T, C> Query<T, C> for E
where
    E: Endpoint + Sync,
    T: DeserializeOwned + 'static,
    C: Client + Sync,
{
    async fn query(&self, client: &C) -> Result<T, ApiError<C::Error>> {
        let url = client.route_url(&self.route()).await?;
        let req = Request::builder().method(self.method()).uri(url.as_str());

        let resp = client.send_request(req, None).await?;

        let status = resp.status();

        // we are assuming all endpoints return JSON for both success and error
        // responses
        let json = if let Ok(json) = serde_json::from_slice(resp.body()) {
            json
        } else {
            return Err(ApiError::server_error(status, resp.body()));
        };

        if !status.is_success() {
            return Err(ApiError::from_json(json));
        }

        serde_json::from_value(json).map_err(ApiError::data_type::<T>)
    }
}
