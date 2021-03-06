use std::collections::HashMap;

use async_trait::async_trait;
use bytes::Bytes;
use derive_builder::Builder;
use http::{Method, Request, Response, StatusCode};
use thiserror::Error;
use url::Url;

use crate::{ApiError, Client};

const MOCK_ROOT_ROUTE: &'static str = "test://test";
const MOCK_ROUTE: &str = "/mock";

/// Mock a response.
#[derive(Debug, Builder)]
pub struct MockResponse {
    /// HTTP method
    #[builder(default = "Method::GET")]
    pub method: Method,

    /// Route
    #[builder(default = "MOCK_ROUTE.to_string()", setter(into))]
    pub route: String,

    /// Response body
    #[builder(default, setter(into))]
    pub body: Vec<u8>,

    /// Response status
    #[builder(default = "StatusCode::OK")]
    pub status: StatusCode,
}

impl MockResponse {
    pub fn builder() -> MockResponseBuilder {
        MockResponseBuilder::default()
    }
}

impl MockResponseBuilder {
    pub fn json(&mut self, value: serde_json::Value) -> &mut MockResponseBuilder {
        let body = serde_json::to_vec(&value).expect("failed to convert json to vec");
        self.body(body)
    }
}

#[derive(Debug, Error)]
#[error("mock client error")]
pub struct MockClientError;

pub struct MockClient {
    response_map: HashMap<(Method, String), MockResponse>,
}

impl MockClient {
    pub fn new() -> Self {
        let response_map = HashMap::new();
        Self { response_map }
    }

    pub fn with_response(response: MockResponse) -> Self {
        let mut client = Self::new();
        client.insert(response);
        client
    }

    pub fn insert(&mut self, response: MockResponse) {
        let request = (response.method.clone(), response.route.to_string());
        self.response_map.insert(request, response);
    }
}

#[async_trait]
impl Client for MockClient {
    type Error = MockClientError;

    async fn route_url(&self, route: &str) -> Result<Url, ApiError<Self::Error>> {
        let url = format!("{}/{}", MOCK_ROOT_ROUTE, route.trim_start_matches("/"));
        Ok(Url::parse(&url).expect("failed to parse url"))
    }

    async fn send_request(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>> {
        let key = (request.method().clone(), request.uri().path().into());

        let mock = self
            .response_map
            .get(&key)
            .expect("no matching request found");

        let resp = Response::builder()
            .status(mock.status)
            .body(mock.body.clone().into())
            .expect("failed to build response");

        Ok(resp)
    }
}
