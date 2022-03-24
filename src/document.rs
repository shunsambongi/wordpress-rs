use async_trait::async_trait;
use http::Method;
use serde::de::DeserializeOwned;
use url::Url;

use crate::{client::Client, error::ApiError, query::Query, request::RequestBuilder};

/// URL to the web page for a resource
pub struct Document(Url);

#[async_trait]
impl<T, C> Query<T, C> for Document
where
    T: DeserializeOwned + 'static,
    C: Client + Sync,
{
    async fn query(&self, client: &C) -> Result<T, ApiError<C::Error>> {
        let url = client.discover_resource(self.0.as_str()).await?;
        RequestBuilder::new()
            .method(Method::GET)
            .url(url)
            .query(client)
            .await
    }
}
