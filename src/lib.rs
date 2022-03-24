pub use crate::{
    client::Client, endpoint::Endpoint, error::ApiError, query::Query, root::RootRoute,
};

mod client;
mod endpoint;
mod error;
mod query;
mod request;
mod root;

#[cfg(feature = "client")]
pub use crate::wordpress::WordPress;
#[cfg(feature = "client")]
mod wordpress;

#[cfg(test)]
mod test;
