#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::undocumented_unsafe_blocks
)]
#![warn(missing_docs)]

//! The `kromer_api` crate provides a strongly typed interface for the [Kromer2]
//! currency server. It omits some features offered by Kromer2 that are not
//! needed, or have better ways to go about using. These are listed in the
//! [omissions](crate#ommissions) section below.
//!
//! ```rust
//! # use kromer_api::Error;
//! # async fn run() -> Result<(), Error> {
//! let client = kromer_api::http::Client::new("https://kromer.reconnected.cc/")?;
//! let motd = client.get_motd().await?;
//!
//! assert_eq!(&motd.public_url, "https://kromer.reconnected.cc/");
//! # Ok(())
//! # }
//!
//! ```
//!
//! # Features
//! At the moment, this crate offers all the HTTP endpoints currently
//! implemented by Kromer2. Most of them are available out of the box, but those
//! that require Kromer2's internal key will require you to enable the
//! `internal` feature flag and create a client using the appropriate function.
//!
//! The websocket API also has support with the `websocket` feature flag. Functionally, it supports
//! everything Kromer2 offers, with some caveats specified in its relevant [documentation](ws).
//!
//! The lookup API will be implemented once Kromer2 has merged support for more endpoints.
//!
//! # Omissions
//! There are some notable things that I've left out of this crate because they
//! are either not needed for the Kromer2 API, or there are better ways to do
//! it.
//!
//! - [Make Address](https://krist.dev/docs/#api-MiscellaneousGroup-MakeV2Address):
//!   Use the [`Address::from`] trait implementation instead. This preforms a series
//!   of expensive hashes, but not as expensive as IO.
//!
//! - [List new names](https://krist.dev/docs/#api-NameGroup-GetNewNames): There are
//!   no new/unpaid names in Kromer2. This endpoint is purely for compatibility's sake.
//!
//! - [`PUT` Name Update Endpoint](https://krist.dev/docs/#api-NameGroup-UpdateNamePUT):
//!   Krist and Kromer2 both have multiple endpoints for updating the metadata of a
//!   [`Name`](model::krist::Name). We provide the `POST` endpoint only rather than have both
//!
//! - Many Krist mining things - Kromer2 does not support earning currency through mining,
//!   but provides many of the values and endpoints pertaining to it for the sake of
//!   compatibility. We don't include this information. If you would still like to harm
//!   the environment, you might consider vanity address mining.
//!
//! [`Address::from`]: model::Address::from<model::PrivateKey>
//! [Kromer2]: https://github.com/ReconnectedCC/kromer2

pub mod http;
pub mod model;

#[cfg(feature = "websocket")]
pub mod ws;

use snafu::Snafu;

/// Errors emitted by the `kromer_api` crate
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum Error {
    #[snafu(display("couldn't parse provide string into URL"))]
    BadUrl { source: url::ParseError },
    /// Emitted when the underlying [`reqwest`] client can't build a request
    #[snafu(display("Failed to build request to"))]
    BadRequest { source: reqwest::Error },
    /// Emitted when there is an issue parsing a `JSON` body received in a
    /// response
    #[snafu(display("Could not parse JSON body into response"))]
    MalformedResponse { source: reqwest::Error },
    /// Emitted when there is an issue communicating with the server itself
    #[snafu(display("Could not dispatch request"))]
    RequestFailed { source: reqwest::Error },
    /// Issues parsing into models
    #[snafu(transparent)]
    ParseError { source: model::ParseError },
    /// Returned by the Kromer API
    #[snafu(transparent)]
    KromerResponse { source: model::KromerError },
    /// Returned by the Krist API
    #[snafu(transparent)]
    KristResponse { source: model::krist::KristError },
    #[cfg(feature = "internal")]
    /// Thrown when creating a [`http::Client`]
    BadInternalKey {
        source: reqwest::header::InvalidHeaderValue,
    },
    /// Errors thrown when working with websockets. See [`ws::WebSocketError`] for more info.
    #[cfg(feature = "websocket")]
    #[snafu(transparent)]
    WebsocketError { source: ws::WebSocketError },
}
