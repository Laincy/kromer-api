//! Functionality for creating and reusing defined endpoints.

use std::fmt::Debug;

use crate::{Error, KromerClient};
use serde::Serialize;

pub mod krist;

/// Methods for endpoints that are paginated
pub trait Paginated {
    #[must_use]
    /// Sets the limit for number of responses. Defaults to 50 and is clamped between 1 and 1000.
    fn limit(self, v: usize) -> Self;

    #[must_use]
    /// Sets the offset for responses. Defaults to 0
    fn offset(self, v: usize) -> Self;
}

/// Shared behavior for all endpoints
#[allow(async_fn_in_trait)]
pub trait Endpoint: Debug + Clone {
    /// The value that we are trying to get as an end result from this API
    type Value;

    /// Sends the endpoint's request to the API
    async fn query(&self, client: &KromerClient) -> Result<Self::Value, Error>;
}

/// Shared behavior for Paginated endpoints
#[allow(async_fn_in_trait)]
pub trait PaginatedEndpoint: Paginated + Endpoint + Serialize {
    /// Queries the endpoint, and adds the recieved count to offset
    async fn query_page(&mut self, client: &KromerClient) -> Result<Self::Value, Error>;
}
