use std::error::Error;

use thiserror::Error;

/// Errors which may occur when using API endpoints.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// The client encountered an error.
    #[error("client error: {}", source)]
    Client {
        /// The client error.
        source: E,
    },

    /// URL failed to parse.
    #[error("failed to parse url: {}", source)]
    UrlParse {
        /// The source of the error.
        #[from]
        source: url::ParseError,
    },

    /// Failed to discover API root route
    #[error("failed to discover root route")]
    RootRouteDiscovery,
}

impl<E> ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// Create an API error from a client specific error.
    pub fn client(source: E) -> Self {
        ApiError::Client { source }
    }
}
