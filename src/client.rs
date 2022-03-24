use std::error::Error;

use async_trait::async_trait;
use bytes::Bytes;
use http::{Method, Request, Response};
use url::Url;

use crate::{error::ApiError, root::RootRoute};

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

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
    async fn route_url(&self, route: &str) -> Result<Url, ApiError<Self::Error>>;

    /// Send an HTTP request
    async fn send_request(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>>;

    /// Discover the API root route for a WordPress instance.
    async fn discover_root_route(
        &self,
        url: impl AsRef<str> + Send + 'async_trait,
    ) -> Result<RootRoute, ApiError<Self::Error>> {
        let re = regex!("<(.*)>; rel=\"https://api.w.org/\"");
        let url = Url::parse(url.as_ref())?;
        link_header_discovery(self, &url, re)
            .await?
            .ok_or_else(|| ApiError::root_route_discovery(url))
            .map(|url| url.into())
    }

    /// Resource discovery.
    async fn discover_resource(
        &self,
        url: impl AsRef<str> + Send + 'async_trait,
    ) -> Result<Url, ApiError<Self::Error>> {
        let re = regex!("<(.*)>; rel=\"alternate\"; type=\"application/json\"");
        let url = Url::parse(url.as_ref())?;
        link_header_discovery(self, &url, re)
            .await?
            .ok_or_else(|| ApiError::resource_discovery(url))
    }
}

async fn link_header_discovery<C>(
    client: &C,
    url: &Url,
    re: &regex::Regex,
) -> Result<Option<Url>, ApiError<C::Error>>
where
    C: Client + ?Sized,
{
    let req = Request::builder()
        .method(Method::HEAD)
        .uri(url.as_str())
        .body(Vec::new())
        .map_err(ApiError::request)?;

    let resp = client.send_request(req).await?;

    for header in resp.headers().get_all("link") {
        let header = if let Ok(header) = header.to_str() {
            header
        } else {
            // Move on if header is not all ascii
            continue;
        };

        let captures = if let Some(captures) = re.captures(header) {
            captures
        } else {
            // Move on if header does not match our pattern
            continue;
        };

        let link = captures.get(1).expect("missing capture group").as_str();

        return Ok(Some(Url::parse(link)?.into()));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("discovery error")]
    struct DiscoveryError;

    struct DiscoveryClient {
        link: String,
    }

    impl DiscoveryClient {
        fn new(link: &str) -> Self {
            Self { link: link.into() }
        }
    }

    #[async_trait]
    impl Client for DiscoveryClient {
        type Error = DiscoveryError;

        async fn route_url(&self, _route: &str) -> Result<Url, ApiError<Self::Error>> {
            unimplemented!()
        }

        async fn send_request(
            &self,
            _request: Request<Vec<u8>>,
        ) -> Result<Response<Bytes>, ApiError<Self::Error>> {
            Ok(Response::builder()
                .header("link", &self.link)
                .body(vec![].into())
                .unwrap())
        }
    }

    #[tokio::test]
    async fn discover_root_route() {
        let client =
            DiscoveryClient::new("<http://example.com/wp-json/>; rel=\"https://api.w.org/\"");

        let root_route = client
            .discover_root_route("http://example.com")
            .await
            .unwrap();

        assert_eq!(root_route.as_str(), "http://example.com/wp-json/");
    }

    #[tokio::test]
    async fn discover_resource() {
        let client =
            DiscoveryClient::new("<http://example.com/wp-json/wp/v2/posts/1>; rel=\"alternate\"; type=\"application/json\"");

        let resource = client
            .discover_resource("http://example.com")
            .await
            .unwrap();

        assert_eq!(
            resource.as_str(),
            "http://example.com/wp-json/wp/v2/posts/1"
        );
    }
}
