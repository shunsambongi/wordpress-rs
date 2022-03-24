use async_trait::async_trait;

use crate::{client::Client, error::ApiError};

/// A trait which represents an asynchronous query.
#[async_trait]
pub trait Query<T, C>
where
    C: Client,
{
    /// Perform the query using the passed client.
    async fn query(&self, client: &C) -> Result<T, ApiError<C::Error>>;
}
