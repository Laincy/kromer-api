#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use reqwest::{Method, Url, header};
use serde::{Deserialize, Serialize};
use tracing::info;

pub mod endpoints;
pub mod model;

#[derive(Debug, thiserror::Error)]
/// A top level error containing all the crate's errors.
pub enum KromerError {
    /// Errors emmitted by the Kromer2 server itself
    #[error("krist error({error}): {message}")]
    Krist {
        /// The error type returned by the Krist API
        error: String,
        /// An additional error message returned by the Krist API
        message: String,
    },
    /// Errors caused by Reqwest
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    /// Errors from URL parsing
    #[error(transparent)]
    Url(#[from] url::ParseError),
    /// Errors from parsing a [`WalletAddr`](model::krist::WalletAddr)
    #[error(transparent)]
    WalletAddrParse(#[from] model::krist::WalletAddrParseError),
    /// Errors from parsing a [`WalletPrivateKey`](model::krist::WalletPrivateKey)
    #[error(transparent)]
    WalletPkParse(#[from] model::krist::WalletPkParseError),
}

/// A client for interacting with the Kromer2 API. See [endpoints] for info on how to
/// use it.
pub struct KromerClient {
    url: Url,
    http: reqwest::Client,
}

impl KromerClient {
    /// Create a new client for the Kromer2 API. This will reuse connections.
    /// # Errors
    /// Errors if the passed in value is not valid.
    pub fn new(url: &str) -> Result<Self, KromerError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        Ok(Self {
            url: Url::parse(url)?,
            http: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()?,
        })
    }

    #[inline]
    pub(crate) async fn get<T>(
        &self,
        endpoint: &str,
        query: Option<impl Serialize + Sized>,
    ) -> Result<T, KromerError>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut req = self.http.request(Method::GET, self.url.join(endpoint)?);

        if let Some(q) = query {
            req = req.query(&q);
        }

        let req = req.build()?;
        info!("GET request sent to {}", req.url());

        self.http
            .execute(req)
            .await?
            .json::<T>()
            .await
            .map_err(KromerError::Http)
    }

    pub(crate) async fn post<T>(
        &self,
        endpoint: &str,
        body: &(impl Serialize + Sized + Sync),
    ) -> Result<T, KromerError>
    where
        T: for<'a> Deserialize<'a>,
    {
        let req = self
            .http
            .request(Method::POST, self.url.join(endpoint)?)
            .json(body)
            .build()?;

        info!("POST request sent to {}", req.url());

        self.http
            .execute(req)
            .await?
            .json::<T>()
            .await
            .map_err(KromerError::Http)
    }
}
