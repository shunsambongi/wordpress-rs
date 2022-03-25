pub use crate::{
    client::Client, document::Document, endpoint::Endpoint, error::ApiError, query::Query,
};

mod client;
mod document;
mod endpoint;
pub mod endpoints;
mod error;
mod query;
mod request;
pub mod root;

#[cfg(feature = "client")]
pub use crate::wordpress::WordPress;
#[cfg(feature = "client")]
mod wordpress;

#[cfg(test)]
mod test;
