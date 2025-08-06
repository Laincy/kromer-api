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
    /// Errors emmitted when parsing wallet addresses
    #[error(transparent)]
    WalletAddrParse(#[from] model::krist::WalletAddrParseError),
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

    pub(crate) async fn request<T>(
        &self,
        method: Method,
        endpoint: &str,
        query: Option<impl Serialize + Sized>,
    ) -> Result<T, KromerError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut req = self.http.request(method, self.url.join(endpoint)?);

        if let Some(q) = query {
            req = req.query(&q);
        }

        let req = req.build()?;
        info!(headers = ?req.headers(), "{} request sent to {}", req.method(), req.url());

        self.http
            .execute(req)
            .await?
            .json::<T>()
            .await
            .map_err(KromerError::Http)
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
        self.request::<T>(Method::GET, endpoint, query).await
    }
}
