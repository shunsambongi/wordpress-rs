use std::error::Error;

use async_trait::async_trait;
use bytes::Bytes;
use http::{request::Builder as RequestBuilder, Response};
use url::Url;

use crate::error::ApiError;

/// A trait representing a client that can communicate with a WordPress
/// instance.
#[async_trait]
pub trait Client {
    /// The errors which may occure for this client.
    type Error: Error + Send + Sync + 'static;

    /// Get the full URL for an API route
    ///
    /// This method should handle instances with or without "pretty permalinks"
    /// enabled.
    fn route_url(&self, route: &str) -> Result<Url, ApiError<Self::Error>>;

    /// Send an HTTP request
    async fn send_request(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>>;
}
