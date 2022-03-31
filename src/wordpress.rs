use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use reqwest::Client as HttpClient;
use thiserror::Error;
use tokio::sync::OnceCell;
use url::Url;

use crate::{client::Client, error::ApiError, root::RootRoute};

/// Asynchronous WordPress client.
pub struct WordPress {
    client: HttpClient,
    site_url: Url,
    root_route: OnceCell<RootRoute>,
}

impl WordPress {
    /// Create a new WordPress client.
    pub fn new(site_url: impl AsRef<str>) -> Result<Self, WordPressError> {
        let client = HttpClient::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(10))
            .build()?;
        let wp = Self {
            client,
            site_url: Url::parse(site_url.as_ref())?,
            root_route: OnceCell::new(),
        };
        Ok(wp)
    }

    /// The root route for the WordPress instance.
    ///
    /// The value will change depending on the permalink structure configured
    /// for the site.
    pub async fn root_route(&self) -> Result<&RootRoute, ApiError<WordPressError>> {
        let result = self
            .root_route
            .get_or_try_init(|| self.discover_root_route(&self.site_url))
            .await;
        Ok(result?)
    }
}

#[async_trait]
impl Client for WordPress {
    type Error = WordPressError;

    async fn route_url(&self, route: &str) -> Result<Url, ApiError<Self::Error>> {
        Ok(self.root_route().await?.join(route))
    }

    async fn send_request(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>> {
        use futures_util::TryFutureExt;
        let call = || async {
            let resp = self.client.execute(request.try_into()?).await?;

            let mut http_resp = Response::builder()
                .status(resp.status())
                .version(resp.version());

            let headers = http_resp.headers_mut().unwrap();
            for (key, value) in resp.headers() {
                match headers.entry(key) {
                    http::header::Entry::Occupied(mut entry) => {
                        entry.append(value.clone());
                    }
                    http::header::Entry::Vacant(entry) => {
                        entry.insert(value.clone());
                    }
                }
            }

            Ok(http_resp.body(resp.bytes().await?)?)
        };
        call().map_err(ApiError::client).await
    }
}

/// Errors that may occur when using the WordPress client.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WordPressError {
    #[error("failed to parse url: {}", source)]
    UrlParse {
        #[from]
        source: url::ParseError,
    },

    #[error("communication with wordpress: {}", source)]
    Communication {
        #[from]
        source: reqwest::Error,
    },

    #[error("`http` error: {}", source)]
    Http {
        #[from]
        source: http::Error,
    },
}

impl From<WordPressError> for ApiError<WordPressError> {
    fn from(err: WordPressError) -> Self {
        ApiError::client(err)
    }
}

#[cfg(test)]
mod tests {
    use http::Request;
    use pretty_assertions::assert_eq;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    use super::*;

    #[tokio::test]
    async fn root_route() {
        let mock_server = MockServer::start().await;

        Mock::given(method("HEAD"))
            .respond_with(ResponseTemplate::new(200).insert_header(
                "link",
                "<http://example.com/wp-json/>; rel=\"https://api.w.org/\"",
            ))
            .mount(&mock_server)
            .await;

        let wordpress = WordPress::new(mock_server.uri()).unwrap();

        let root = wordpress.root_route().await.unwrap();

        assert_eq!(root.as_str(), "http://example.com/wp-json/");
    }

    #[tokio::test]
    async fn send_request() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("bob loblaw"))
            .mount(&mock_server)
            .await;

        let wordpress = WordPress::new(mock_server.uri()).unwrap();

        let req = Request::builder()
            .method("GET")
            .uri(mock_server.uri())
            .body(Vec::new())
            .unwrap();

        let resp = wordpress.send_request(req).await.unwrap();

        assert_eq!(resp.body(), "bob loblaw");
    }

    #[tokio::test]
    async fn duplicate_headers() {
        let mock_server = MockServer::start().await;

        Mock::given(method("HEAD"))
            .respond_with(
                ResponseTemplate::new(200)
                    .append_header(
                        "link",
                        "<http://example.com/wp-json/>; rel=\"https://api.w.org/\"",
                    )
                    .append_header(
                        "link",
                        r#"<http://example.com/wp-json/wp/v2/posts/1>; rel="alternate"; type="application/json""#,
                    )
                    .append_header(
                        "link",
                        r#"<http://example.com/?p=1>; rel="shortlink""#,
                    )
            )
            .mount(&mock_server)
            .await;

        let wordpress = WordPress::new(mock_server.uri()).unwrap();

        let req = Request::builder()
            .method("HEAD")
            .uri(mock_server.uri())
            .body(Vec::new())
            .unwrap();

        let resp = wordpress.send_request(req).await.unwrap();

        let links: Vec<_> = resp.headers().get_all("link").into_iter().collect();
        assert_eq!(links.len(), 3);
    }
}
