use std::error::Error;

use async_trait::async_trait;
use http::{Method, Request};
use serde::de::DeserializeOwned;
use url::Url;

use crate::{client::Client, query::Query, ApiError};

#[derive(Default)]
pub struct RequestBuilder {
    method: Option<Method>,
    url: Option<Url>,
    body: Option<Vec<u8>>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn method(&mut self, method: Method) -> &mut Self {
        self.method = Some(method);
        self
    }

    pub fn url(&mut self, url: Url) -> &mut Self {
        self.url = Some(url);
        self
    }

    #[allow(dead_code)]
    pub fn body(&mut self, body: impl Into<Vec<u8>>) -> &mut Self {
        self.body = Some(body.into());
        self
    }

    pub fn build<E>(&self) -> Result<Request<Vec<u8>>, ApiError<E>>
    where
        E: Error + Sync + Send,
    {
        let mut builder = Request::builder();
        if let Some(method) = self.method.clone() {
            builder = builder.method(method);
        }
        if let Some(url) = self.url.clone() {
            builder = builder.uri(url.as_str());
        }
        let request = if let Some(body) = self.body.clone() {
            builder.body(body)
        } else {
            builder.body(Vec::new())
        };

        request.map_err(ApiError::request)
    }
}

#[async_trait]
impl<T, C> Query<T, C> for RequestBuilder
where
    T: DeserializeOwned,
    C: Client + Sync,
{
    async fn query(&self, client: &C) -> Result<T, ApiError<C::Error>> {
        let req = self.build()?;
        let resp = client.send_request(req).await?;

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
