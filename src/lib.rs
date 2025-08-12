#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![warn(missing_docs)]

//! The `kromer-api` crate provides a strongly typed interface for the [Kromer2] currency
//! server. It omits some features offered by Kromer2 that are not needed, or have
//! better ways to go about using. These are listed in the [ommissions](crate#ommissions) section below.
//!
//! ```rust
//! # use kromer_api::endpoints::krist::GetMotdEp;
//! # use kromer_api::KromerClient;
//! # use kromer_api::endpoints::Endpoint;
//! # use kromer_api::Error;
//! # async fn run() -> Result<(), Error> {
//! let client = KromerClient::new("https://kromer.reconnected.cc/")?;
//! let motd = GetMotdEp::new().query(&client).await?;
//!
//! assert_eq!(&motd.public_url, "https://kromer.reconnected.cc/");
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//! At the moment, this crate offers all the HTTP endpoints currently implemented
//! by Kromer2. Most of them are available out of the box, but those in
//! the [`endpoints::internal`] module will require you to enable the
//! `internal` feature flag and create a client using [`KromerClient::new_internal`]
//!
//! Support for the Kromer2 websocket API is planned and in development.
//! The lookup API will be implemented once Kromer2 has merged support
//! for more endpoints.
//!
//! # Ommissions
//! There are some notable things that I've left out of this crate becuause
//! they are eitheir not needed for the Kromer2 API, or there are better
//! ways to do it.
//!
//! - [Make V2 Address](https://krist.dev/docs/#api-MiscellaneousGroup-MakeV2Address):
//!   Use the [`Address::from`] trait implementation instead. This preforms a series 
//!   of expensive hashes, but not as expensive as IO.
//! - [List new names](https://krist.dev/docs/#api-NameGroup-GetNewNames): There are
//!   no new/unpaid names in Kromer2. This endpoint is purely for compatibility's sake.
//! - [`PUT` Name Update Endpoint](https://krist.dev/docs/#api-NameGroup-UpdateNamePUT):
//!   Krist and Kromer2 both have multiple endpoints for updating the metadata of a
//!   [`Name`](model::krist::Name). We provide the `POST` endpoint only rather than have both
//! - Many Krist mining things - Kromer2 does not support earning currency through mining,
//!   but provides many of the values and endpoints pertaining to it for the sake of
//!   compatibility. We don't include this information. If you would still like to harm
//!   the environment, you might consider vanity address mining.
//!
//! [`Address::from`]: model::krist::Address::from<model::krist::PrivateKey>
//! [Kromer2]: https://github.com/ReconnectedCC/kromer2

use reqwest::header;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::{info, warn};

use crate::model::RawKromerError;
use crate::model::krist::RawKristError;

pub mod endpoints;
pub mod model;

/// Errors emmitted by the `kromer-api` Crate
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum Error {
    /// Emmitted when parsing a URL fails
    #[snafu(display(r#"Could not parse input "{url}" into a valid URL"#))]
    InvalidUrl {
        source: url::ParseError,
        url: String,
    },
    /// Emmitted when the underlying [`reqwest`] client can't build a request
    #[snafu(display("Failed to build request to"))]
    BadRequest { source: reqwest::Error },
    /// Emmitted when there is an issue communicating with the server itself
    #[snafu(display("Could not dispatch request"))]
    RequestFailed { source: reqwest::Error },
    /// Emmitted when there is an issue parsing a JSON body recieved in a response into structured
    /// data
    #[snafu(display("Could not parse JSON body into response"))]
    MalformedResponse { source: reqwest::Error },
    /// Returned by the Krist API
    #[snafu(transparent)]
    KristResponse { source: model::krist::KristError },
    #[snafu(transparent)]
    /// Returned by the Kromer API
    KromerResponse { source: model::KromerError },
    /// Issue when parsing a name
    #[snafu(transparent)]
    NameParse {
        source: model::krist::NameParseError,
    },
    /// Issue when parsing a wallet [`Address`](model::krist::Address)
    #[snafu(transparent)]
    WalletParse {
        source: model::krist::WalletParseError,
    },
    #[cfg(feature = "internal")]
    /// Thrown when creating a [`KromerClient`]
    BadInternalKey {
        source: reqwest::header::InvalidHeaderValue,
    },
}

#[derive(Debug, Deserialize)]
struct KromerExtractHelper<T> {
    pub data: T,
}

/// A client for querying the `Kromer2` Api
#[derive(Debug, Clone)]
pub struct KromerClient<T>
where
    T: Send + Sync,
{
    http: Client,
    url: Url,
    _phantom: PhantomData<T>,
}

/// A marker denoting a standard client
#[derive(Debug, Clone, Copy)]
pub struct Basic;

/// A marker type denoting a client that can query internal endpoints
#[derive(Debug, Clone, Copy)]
pub struct Privileged;

impl KromerClient<Basic> {
    /// Create a new client for the Kromer2 API. This will reuse connections.
    /// # Errors
    /// Errors if the passed in value is not valid.
    /// # Panics
    /// Panics if we cannot construct the client for an unknown reason. Chances are, if this occurs
    /// it is irrecoverable and an issue at the crate level
    pub fn new(url: &str) -> Result<Self, Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Self {
            url: Url::parse(url).context(InvalidUrlSnafu {
                url: url.to_string(),
            })?,
            #[allow(clippy::panic)]
            http: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()
                .expect("HTTP is fucked, stop trying"),

            _phantom: PhantomData,
        };

        info!("Initialized client for {}", client.url);

        Ok(client)
    }
}

#[derive(Debug, Deserialize)]
struct RawKromerWrapper {
    error: RawKromerError,
}

#[cfg(feature = "internal")]
impl KromerClient<Privileged> {
    /// Create a new client for the Kromer2 API with an internal key. This will reuse connections.
    /// # Errors
    /// Errors if the passed in value is not valid.
    /// # Panics
    /// Panics if we cannot construct the client for an unknown reason. Chances are, if this occurs
    /// it is irrecoverable and an issue at the crate level
    pub fn new_internal(url: &str, key: &str) -> Result<Self, Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Kromer-Key",
            header::HeaderValue::from_str(key).context(BadInternalKeySnafu {})?,
        );

        let client = Self {
            url: Url::parse(url).context(InvalidUrlSnafu {
                url: url.to_string(),
            })?,
            #[allow(clippy::panic)]
            http: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()
                .expect("HTTP is fucked, stop trying"),

            _phantom: PhantomData,
        };

        info!("Initialized client for {}", client.url);

        Ok(client)
    }

    pub(crate) async fn internal_get<T>(&self, endpoint: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de> + Sized,
    {
        let url = self.url.join(endpoint).expect("Passed a bad URL");

        let req = self.http.get(url).build().context(BadRequestSnafu)?;

        info!("Sent a GET request to {}", req.url());

        let response = self
            .http
            .execute(req)
            .await
            .context(RequestFailedSnafu {})?;

        if !response.status().is_success() {
            warn!("Recieved HTTP response {}", response.status());

            let err = response
                .json::<RawKromerWrapper>()
                .await
                .context(MalformedResponseSnafu)?
                .error
                .parse()
                .expect_err("Cannot return Ok");
            return Err(Error::KromerResponse { source: err });
        }

        response.json::<T>().await.context(MalformedResponseSnafu)
    }

    pub(crate) async fn internal_post<T>(
        &self,
        endpoint: &str,
        body: &(impl Serialize + Sized + Sync),
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de> + Sized,
    {
        let url = self.url.join(endpoint).expect("Passed a bad URL");

        let req = self
            .http
            .post(url)
            .json(body)
            .build()
            .context(BadRequestSnafu)?;

        info!("Sent a POST request to {}", req.url());
        let response = self
            .http
            .execute(req)
            .await
            .context(RequestFailedSnafu {})?;

        if !response.status().is_success() {
            warn!("Recieved HTTP response {}", response.status());

            let err = response
                .json::<RawKromerWrapper>()
                .await
                .context(MalformedResponseSnafu)?
                .error
                .parse()
                .expect_err("Cannot return Ok");
            return Err(Error::KromerResponse { source: err });
        }

        response.json::<T>().await.context(MalformedResponseSnafu)
    }
}

impl<M: Clone + Debug + Send + Sync> KromerClient<M> {
    #[allow(dead_code)]
    pub(crate) async fn get<T>(&self, endpoint: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de> + Sized,
    {
        // Safety:
        // We can panic here because if this join fails, the crate itself is passing a bad URL in
        let url = self.url.join(endpoint).expect("Passed a bad URL");

        let req = self.http.get(url).build().context(BadRequestSnafu)?;

        info!("Sent a GET request to {}", req.url());
        let response = self
            .http
            .execute(req)
            .await
            .context(RequestFailedSnafu {})?;

        if !response.status().is_success() {
            warn!("Recieved HTTP response {}", response.status());

            let err = response
                .json::<RawKromerWrapper>()
                .await
                .context(MalformedResponseSnafu)?
                .error
                .parse()
                .expect_err("Cannot return ok");

            return Err(Error::KromerResponse { source: err });
        }

        Ok(response
            .json::<KromerExtractHelper<T>>()
            .await
            .context(MalformedResponseSnafu)?
            .data)
    }

    pub(crate) async fn krist_get<T>(
        &self,
        endpoint: &str,
        query: Option<impl Serialize + Sized>,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de> + Sized,
    {
        // Safety:
        // We can panic here because if this join fails, the crate itself is passing a bad URL in
        let url = self.url.join(endpoint).expect("Passed a bad URL");

        let mut req = self.http.get(url.clone());

        if let Some(q) = query {
            req = req.query(&q);
        }

        let req = req.build().context(BadRequestSnafu {})?;

        info!("Sent a GET request to {}", req.url());
        let response = self
            .http
            .execute(req)
            .await
            .context(RequestFailedSnafu {})?;

        if !response.status().is_success() {
            warn!("Recieved HTTP response {}", response.status());
            let err = response
                .json::<RawKristError>()
                .await
                .context(MalformedResponseSnafu)?
                .parse()
                .expect_err("Cannot return ok");

            return Err(Error::KristResponse { source: err });
        }
        response.json::<T>().await.context(MalformedResponseSnafu)
    }

    pub(crate) async fn krist_post<T>(
        &self,
        endpoint: &str,
        body: &(impl Serialize + Sized + Sync),
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de> + Sized,
    {
        // Safety:
        // We can panic here because if this join fails, the crate itself is passing a bad URL in
        let url = self.url.join(endpoint).expect("Passed a bad URL");

        let req = self
            .http
            .post(url)
            .json(body)
            .build()
            .context(BadRequestSnafu {})?;

        info!("Sent a GET request to {}", req.url());
        let response = self
            .http
            .execute(req)
            .await
            .context(RequestFailedSnafu {})?;

        if !response.status().is_success() {
            warn!("Recieved HTTP response {}", response.status());

            let err = response
                .json::<RawKristError>()
                .await
                .context(MalformedResponseSnafu)?
                .parse()
                .expect_err("Cannot return ok");

            return Err(Error::KristResponse { source: err });
        }

        response.json::<T>().await.context(MalformedResponseSnafu)
    }
}
