use std::{borrow::Cow, collections::HashMap};

use async_trait::async_trait;
use bytes::Bytes;
use derive_builder::Builder;
use http::{request::Builder as RequestBuilder, Method, Response, StatusCode};
use thiserror::Error;
use url::Url;

use crate::{ApiError, Client, Endpoint};

const MOCK_ROOT_ROUTE: &'static str = "test://test";
const MOCK_ROUTE: &str = "/mock";

pub struct MockEndpoint;

impl Endpoint for MockEndpoint {
    fn method(&self) -> Method {
        Method::GET
    }

    fn route(&self) -> Cow<'static, str> {
        MOCK_ROUTE.into()
    }
}

/// Mock a response.
#[derive(Debug, Builder)]
pub struct MockResponse {
    /// HTTP method
    #[builder(default = "Method::GET")]
    pub method: Method,

    /// Route
    #[builder(default = "MOCK_ROUTE")]
    pub route: &'static str,

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
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>> {
        let body = if let Some(body) = body {
            body
        } else {
            Vec::new()
        };

        let req = request.body(body).expect("failed to build request");

        let key = (req.method().clone(), req.uri().path().into());

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
