#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use reqwest::header;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use tracing::{info, warn};

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
}

/// A client for querying the `Kromer2` Api
#[derive(Debug, Clone)]
pub struct KromerClient {
    http: Client,
    url: Url,
}

impl KromerClient {
    /// Create a new client for the Kromer2 API. This will reuse connections.
    /// # Errors
    /// Errors if the passed in value is not valid.
    /// # Panics
    /// Panics if we cannot construct the client for an unknown reason. Chances are, if this occurs
    /// this is irrecoverable anyways
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
        };

        info!("Initialized client for {}", client.url);

        Ok(client)
    }

    #[allow(dead_code)]
    pub(crate) async fn get<T>(
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

    pub(crate) async fn post<T>(
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
