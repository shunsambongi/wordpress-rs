pub use crate::{client::Client, error::ApiError, root::RootRoute};

mod client;
mod error;
mod root;

#[cfg(feature = "client")]
pub use crate::wordpress::WordPress;
#[cfg(feature = "client")]
mod wordpress;
