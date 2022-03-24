use std::borrow::Cow;

use async_trait::async_trait;
use http::Method;
use serde::de::DeserializeOwned;

use crate::{client::Client, query::Query, request::RequestBuilder, ApiError};

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
        RequestBuilder::new()
            .method(self.method())
            .url(url)
            .query(client)
            .await
    }
}

pub mod prelude {
    pub use std::borrow::Cow;

    pub use http::Method;

    pub use super::Endpoint;
}
