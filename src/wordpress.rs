use async_trait::async_trait;
use bytes::Bytes;
use http::{request::Builder as RequestBuilder, Response};
use thiserror::Error;
use url::Url;

use crate::{client::Client, error::ApiError};

pub struct WordPress {}

#[async_trait]
impl Client for WordPress {
    type Error = WordPressError;

    fn route_url(&self, route: &str) -> Result<Url, ApiError<Self::Error>> {
        todo!()
    }

    async fn send_request(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>> {
        todo!()
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WordPressError {}
