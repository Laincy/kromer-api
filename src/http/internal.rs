use super::ClientMarkerSealed;
use crate::{
    BadInternalKeySnafu, BadRequestSnafu, BadUrlSnafu, Error, MalformedResponseSnafu,
    http::{Client, kromer::KromerResponse},
    model::{Address, PrivateKey, Wallet},
};
use reqwest::{Request, header};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::marker::PhantomData;
use tracing::info;
use url::Url;
use uuid::Uuid;

/// A marker type denoting a [`Client`](super::Client) that can use internal endpoints
pub struct Priviliged;

impl ClientMarkerSealed for Priviliged {}

impl Client<Priviliged> {
    /// Create a new client for the Kromer2 API. This will reuse connections.
    ///
    /// # Errors
    /// Errors if `url` is not a valid [`Url`]
    ///
    /// See [`Error`] for more info
    ///
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
            header::HeaderValue::from_str(key).context(BadInternalKeySnafu)?,
        );

        let client = Self {
            url: Url::parse(url).context(BadUrlSnafu)?,

            // Safety:
            // We can expect here because this should *never* fail uness something is fucked
            #[allow(clippy::expect_used)]
            http: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()
                .expect("HTTP is fucked, stop trying"),

            _marker: PhantomData,
        };

        info!("Initialized client for {}", client.url);

        Ok(client)
    }

    async fn internal_query<T>(&self, req: Request) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let resp = self.query(req).await?;

        if !resp.status().is_success() {
            resp.json::<KromerResponse<i32>>()
                .await
                .context(MalformedResponseSnafu)?
                .extract()?;

            unreachable!()
        }

        resp.json::<T>().await.context(MalformedResponseSnafu)
    }

    async fn internal_get<T>(&self, endpoint: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.url.join(endpoint).context(BadUrlSnafu)?;
        let req = self.http.get(url).build().context(BadRequestSnafu)?;

        self.internal_query(req).await
    }

    async fn internal_post<T>(
        &self,
        endpoint: &str,
        body: impl Serialize + Send + Sync + Sized,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.url.join(endpoint).context(BadUrlSnafu)?;

        let req = self
            .http
            .post(url)
            .json(&body)
            .build()
            .context(BadRequestSnafu)?;

        self.internal_query(req).await
    }

    /// Gets all [`Wallets`](Wallet) owned by `id`, along with the 32 byte hash of
    /// {address}{private key} for each wallet.
    ///
    /// # Errors
    /// Errors if there is a network error or you are unauthorized
    ///
    /// See [`Error`] for more info
    pub async fn get_wallet_internal(&self, id: &Uuid) -> Result<Vec<(Wallet, [u8; 32])>, Error> {
        let url = format!("/api/_internal/wallet/by-player/{id}");

        let res = self.internal_get::<UuidListRes>(&url).await?.wallet;

        Ok(res.into_iter().map(|v| (v.wallet, v.pk)).collect())
    }

    /// Creates a [`Wallet`] linked to `id` and returns an [`Address`] and [`PrivateKey`] tuple
    ///
    /// # Errors
    /// Can error if there is there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn create_wallet(&self, id: &Uuid) -> Result<(Address, PrivateKey), Error> {
        let res = self
            .internal_post::<CreateWalletRes>("/api/_internal/wallet/create", &[("uuid", id)])
            .await?;

        Ok((res.address, res.privatekey))
    }

    /// Adds `amount` kromer to the wallet `addr` points to
    ///
    /// # Errors
    /// Errors if the wallet does not exist or there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn give_money(&self, addr: &Address, amount: Decimal) -> Result<Wallet, Error> {
        let body = GiveMoneyBody { addr, amount };

        Ok(self
            .internal_post::<WalletRes>("/api/_internal/wallet/give-money", body)
            .await?
            .wallet)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct WalletRes {
    wallet: Wallet,
}

#[derive(Debug, Deserialize, Clone)]
struct InternalWalletRes {
    #[serde(flatten)]
    wallet: Wallet,
    #[serde(rename = "private_key")]
    pk: [u8; 32],
}

#[derive(Debug, Deserialize, Clone)]
struct UuidListRes {
    wallet: Vec<InternalWalletRes>,
}

#[derive(Debug, Deserialize)]
struct CreateWalletRes {
    privatekey: PrivateKey,
    address: Address,
}

#[derive(Debug, Serialize)]
struct GiveMoneyBody<'a> {
    #[serde(rename = "address")]
    addr: &'a Address,
    amount: Decimal,
}
