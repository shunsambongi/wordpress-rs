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

#[cfg(test)]
mod tests {
    use http::StatusCode;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::test::{MockClient, MockEndpoint, MockResponse};

    type Json = serde_json::Value;

    #[tokio::test]
    async fn non_json_response() {
        let response = MockResponse::builder().body("not json").build().unwrap();
        let client = MockClient::with_response(response);

        let result: Result<Json, _> = MockEndpoint.query(&client).await;

        let err = result.expect_err("expected ApiError::WordPressInternal");
        if let ApiError::WordPressInternal { status, data } = err {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(
                String::from_utf8(data).expect("unexpected data"),
                "not json"
            );
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[tokio::test]
    async fn empty_response() {
        let response = MockResponse::builder().build().unwrap();
        let client = MockClient::with_response(response);

        let result: Result<Json, _> = MockEndpoint.query(&client).await;

        let err = result.expect_err("expected ApiError::WordPressInternal");
        if let ApiError::WordPressInternal { status, data } = err {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(String::from_utf8(data).expect("unexpected data"), "");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    /// Non-JSON response error takes precedence over error status.
    #[tokio::test]
    async fn non_json_error_status_response() {
        let response = MockResponse::builder()
            .status(StatusCode::NOT_FOUND)
            .body("not json")
            .build()
            .unwrap();
        let client = MockClient::with_response(response);

        let result: Result<Json, _> = MockEndpoint.query(&client).await;

        let err = result.expect_err("expected ApiError::WordPressInternal");
        if let ApiError::WordPressInternal { status, data } = err {
            assert_eq!(status, StatusCode::NOT_FOUND);
            assert_eq!(
                String::from_utf8(data).expect("unexpected data"),
                "not json"
            );
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[tokio::test]
    async fn error_response() {
        let body = json!({
            "code": "rest_post_invalid_id",
            "message": "Invalid post ID.",
            "data": {
                "status": 404_u16
            }
        });
        let body = serde_json::to_vec(&body).unwrap();
        let response = MockResponse::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body)
            .build()
            .unwrap();
        let client = MockClient::with_response(response);

        let result: Result<Json, _> = MockEndpoint.query(&client).await;

        let err = result.expect_err("expected ApiError::WordPress");
        if let ApiError::WordPress { message, code, .. } = err {
            assert_eq!(code, "rest_post_invalid_id");
            assert_eq!(message, "Invalid post ID.");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[tokio::test]
    async fn unrecognized_error_response() {
        let body_obj = json!({ "bob": "loblaw" });
        let body = serde_json::to_vec(&body_obj).unwrap();

        let response = MockResponse::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body)
            .build()
            .unwrap();
        let client = MockClient::with_response(response);

        let result: Result<Json, _> = MockEndpoint.query(&client).await;

        let err = result.expect_err("expected ApiError::WordPressUnrecognized");
        if let ApiError::WordPressUnrecognized { json } = err {
            assert_eq!(json, body_obj);
        } else {
            panic!("unexpected error: {}", err);
        }
    }
}
