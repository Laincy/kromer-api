//! Types for interacting with Kromer2's HTTP API

use rust_decimal::Decimal;
pub use util::*;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_REPO: &str = env!("CARGO_PKG_REPOSITORY");

use crate::{
    BadRequestSnafu, BadUrlSnafu, Error, MalformedResponseSnafu, RequestFailedSnafu,
    model::{
        Address, PrivateKey, Wallet,
        krist::{
            KristError, Motd, Name, NameInfo, NamePage, SameWalletTransferSnafu, Transaction,
            TransactionPage, WalletPage,
        },
    },
};
use reqwest::{Request, Response, header};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, ensure};
use std::marker::PhantomData;
use tracing::{trace, warn};
use url::Url;
use uuid::Uuid;

#[cfg(feature = "websocket")]
mod ws;

#[cfg(feature = "internal")]
pub use internal::*;
#[cfg(feature = "internal")]
mod internal;

mod krist;
mod kromer;
mod util;

pub(crate) use krist::RawKristError;

use krist::{
    AuthRequest, AuthRes, AvailRes, CostRes, ListTransactionsQuery, MakeTransactionBody, NameRes,
    RegisterBody, SupplyRes, TransactionRes, TransferBody, UpdateBody,
};
use kromer::KromerResponse;

/// An HTTP client for calling the Kromer2 API. Reuses connections and parses
/// responses into idiomatic rust types.
pub struct Client<M: ClientMarker> {
    url: url::Url,
    http: reqwest::Client,
    _marker: PhantomData<M>,
}

impl Client<Basic> {
    /// Create a new client for the Kromer2 API. This will reuse connections.
    ///
    /// # Errors
    /// Errors if `url` is not a valid [`Url`]
    ///
    /// See [`Error`] for more info
    ///
    /// # Panics
    /// Panics if we cannot construct the client for an unknown reason. Chances
    /// are, if this occurs it is irrecoverable and an issue at the crate level
    pub fn new(url: &str) -> Result<Self, Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let user_agent = format!("{PKG_NAME}/{PKG_VERSION} ({PKG_REPO})");

        let client = Self {
            url: Url::parse(url).context(BadUrlSnafu)?,

            // Safety:
            // We can expect here because this should *never* fail unless something is fucked
            #[allow(clippy::expect_used)]
            http: reqwest::ClientBuilder::new()
                .user_agent(user_agent)
                .default_headers(headers)
                .build()
                .expect("HTTP is fucked, stop trying"),

            _marker: PhantomData,
        };

        trace!("Initialized client for {}", client.url);

        Ok(client)
    }
}

impl<M: ClientMarker> Client<M> {
    /// General query behavior
    async fn query(&self, req: Request) -> Result<Response, Error> {
        trace!("sending a {} request to {}", req.method(), req.url());
        let response = self.http.execute(req).await.context(RequestFailedSnafu)?;

        let status = response.status();

        if !status.is_success() {
            warn!("got HTTP code {} from {}", status, response.url());
        }

        Ok(response)
    }

    /// Get requests against the Kromer2 API
    async fn get<T>(&self, endpoint: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.url.join(endpoint).context(BadUrlSnafu)?;

        let req = self.http.get(url).build().context(BadRequestSnafu)?;

        Ok(self
            .query(req)
            .await?
            .json::<KromerResponse<T>>()
            .await
            .context(MalformedResponseSnafu)?
            .extract()?)
    }

    async fn krist_get<T>(
        &self,
        endpoint: &str,
        query: Option<(impl Serialize + Send + Sync + Sized)>,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.url.join(endpoint).context(BadUrlSnafu)?;

        let req = self
            .http
            .get(url)
            .query(&query)
            .build()
            .context(BadRequestSnafu)?;

        let response = self.query(req).await?;

        if !response.status().is_success() {
            response
                .json::<RawKristError>()
                .await
                .context(MalformedResponseSnafu)?
                .parse()?;

            // Above will always return an Err
            unreachable!()
        }

        response.json::<T>().await.context(MalformedResponseSnafu)
    }

    async fn krist_post<T>(
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

        let response = self.query(req).await?;

        if !response.status().is_success() {
            response
                .json::<RawKristError>()
                .await
                .context(MalformedResponseSnafu)?
                .parse()?;

            // Above will always return an Err
            unreachable!()
        }

        response.json::<T>().await.context(MalformedResponseSnafu)
    }

    /// Fetches all [`Wallets`](Wallet) attached to a `Minecraft` `UUID`
    /// # Errors
    /// Errors if there is no user with a `UUID` of `id` found by Kromer2, or
    /// there is some other network issue.
    ///
    /// See [`Error`] for more info
    pub async fn get_wallet_uuid(&self, id: &Uuid) -> Result<Vec<Wallet>, Error> {
        let url = format!("/api/v1/wallet/by-uuid/{id}");
        self.get(&url).await
    }

    /// Fetches all [`Wallets`](Wallet) attached to a `Minecraft` `username`
    /// # Errors
    /// Errors if there is no user with `name` found by Kromer2, or there is
    /// some other network issue.
    ///
    /// See [`Error`] for more info
    pub async fn get_wallet_name(&self, name: &str) -> Result<Vec<Wallet>, Error> {
        let url = format!("/api/v1/wallet/by-name/{name}");
        self.get(&url).await
    }

    /// Fetches the [`Motd`] from the Krist API
    ///
    /// # Errors
    /// See [`Error`] for more info
    pub async fn get_motd(&self) -> Result<Motd, Error> {
        self.krist_get("/api/krist/motd", None::<()>).await
    }

    /// Fetches a [`Wallet`] from the Krist API
    ///
    /// # Errors
    /// Errors if `addr` does not exist or there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn get_wallet_addr(&self, addr: &Address) -> Result<Wallet, Error> {
        let url = format!("/api/krist/addresses/{addr}");
        Ok(self
            .krist_get::<krist::GetAddrRes>(&url, None::<()>)
            .await?
            .address
            .wallet)
    }

    /// Fetches a [`Wallet`] from the Krist API as a `tuple` with the number of
    /// names that wallet owns
    ///
    /// # Errors
    /// Errors if `addr` does not exist or there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn get_wallet_addr_wnames(&self, addr: &Address) -> Result<(Wallet, u32), Error> {
        let url = format!("/api/krist/addresses/{addr}?fetchNames=true");
        let res = self
            .krist_get::<krist::GetAddrRes>(&url, None::<()>)
            .await?
            .address;

        Ok((res.wallet, res.names))
    }

    /// Fetches a [`WalletPage`] from the Krist API
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn list_wallets(&self, page: Option<&Paginator>) -> Result<WalletPage, Error> {
        self.krist_get("/api/krist/addresses", page).await
    }

    /// Fetches the richest wallets as a [`WalletPage`] from the Krist API
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn list_rich(&self, page: Option<&Paginator>) -> Result<WalletPage, Error> {
        self.krist_get("/api/krist/addresses/rich", page).await
    }

    /// Fetches an address' recent transactions as a [`TransactionPage`]
    ///
    /// # Arguments
    /// * `addr` - The [`Address`] who's transactions you would like to list
    /// * `mined` - Whether to include transactions of type "Mined" in the response
    ///
    /// # Errors
    /// Errors if `addr` does not exist, or if there is a network issue.
    ///
    /// See [`Error`] for more info
    pub async fn recent_wallet_transactions(
        &self,
        addr: &Address,
        mined: bool,
        page: Option<&Paginator>,
    ) -> Result<TransactionPage, Error> {
        let url = format!(
            "/api/krist/addresses/{}/transactions?excludeMined={}",
            addr, !mined
        );

        self.krist_get(&url, page).await
    }

    /// Fetches the names owned by an address as a [`NamePage`]
    ///
    /// # Errors
    /// Errors if `addr` does not exist or there is a network issue.
    ///
    /// See [`Error`] for more info
    pub async fn list_wallet_names(
        &self,
        addr: &Address,
        page: Option<&Paginator>,
    ) -> Result<NamePage, Error> {
        let url = format!("/api/krist/addresses/{addr}/names");

        self.krist_get(&url, page).await
    }

    /// Checks if a [`PrivateKey`] corresponds with an address on the Kromer2
    /// server, if it does it returns the address, if not it creates a new
    /// address and returns it.
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn login(&self, pk: &PrivateKey) -> Result<Address, Error> {
        let body = AuthRequest { pk: pk.inner() };

        Ok(self
            .krist_post::<AuthRes>("/api/krist/login", body)
            .await?
            .address)
    }

    /// Checks the amount of Kromer in circulation
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn supply(&self) -> Result<Decimal, Error> {
        Ok(self
            .krist_get::<SupplyRes>("/api/krist/supply", None::<()>)
            .await?
            .money_supply)
    }

    /// Fetches [`NameInfo`] from the Krist API
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn get_name(&self, name: &Name) -> Result<NameInfo, Error> {
        let url = format!("/api/krist/names/{name}");

        Ok(self.krist_get::<NameRes>(&url, None::<()>).await?.name)
    }

    /// Fetches a [`NamePage`] from the Krist API
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn list_names(&self, page: Option<&Paginator>) -> Result<NamePage, Error> {
        self.krist_get("/api/krist/names", page).await
    }

    /// Gets the cost to buy a [`Name`] from the Krist API
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn name_cost(&self) -> Result<Decimal, Error> {
        Ok(self
            .krist_get::<CostRes>("/api/krist/names/cost", None::<()>)
            .await?
            .name_cost)
    }

    /// Checks if a [`Name`] is available to buy
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn check_name(&self, name: &Name) -> Result<bool, Error> {
        let url = format!("/api/krist/names/check/{name}");

        Ok(self
            .krist_get::<AvailRes>(&url, None::<()>)
            .await?
            .available)
    }

    /// Registers a [`Name`], will error if it fails
    ///
    /// # Errors
    /// Errors if the name is unavailable or the pk links to a wallet with
    /// insufficient funds.
    ///
    /// See [`Error`] for more info
    pub async fn register_name(&self, name: &Name, pk: &PrivateKey) -> Result<(), Error> {
        let url = format!("/api/krist/names/{name}");

        let body = RegisterBody { privatekey: pk };

        self.krist_post::<()>(&url, body).await
    }

    /// Transfers a name to another address
    ///
    /// # Arguments
    /// * `name` - The [`Name`] that you want to transfer
    /// * `addr` - The [`Address`] that you would like to transfer the name to
    /// * `pk` - The [`PrivateKey`] of the address that currently owns the name
    ///
    /// # Errors
    /// Will error if `name` does not belong to the address pointed to by `pk`,
    /// or if there is a network issue.
    ///
    /// See [`Error`] for more info
    pub async fn transfer_name(
        &self,
        name: &Name,
        addr: &Address,
        pk: &PrivateKey,
    ) -> Result<NameInfo, Error> {
        let url = format!("/krist/api/names/{name}/transfer");

        let body = TransferBody {
            address: addr,
            privatekey: pk,
        };

        Ok(self.krist_post::<NameRes>(&url, body).await?.name)
    }

    /// Updates a name, returning the updated [`NameInfo`]
    ///
    /// # Arguments
    /// * `name` - The [`Name`] to update
    /// * `meta` - The text to set the `metadata` to. If `None` it will delete the `metadata`
    /// * `pk` - The [`PrivateKey`] that owns `name`
    ///
    /// # Errors
    /// Will error does not exist or belong to `pk`, or if there is a network
    /// issue.
    ///
    /// See [`Error`] for more info
    pub async fn update_name(
        &self,
        name: &Name,
        meta: Option<&str>,
        pk: &PrivateKey,
    ) -> Result<NameInfo, Error> {
        let url = format!("/api/krist/names/{name}/update");

        let body = UpdateBody {
            privatekey: pk,
            a: meta,
        };

        self.krist_post::<NameInfo>(&url, body).await
    }

    /// Lists transactions in order from oldest to newest as a
    /// [`TransactionPage`]
    ///
    /// # Arguments
    /// * `page` -  The [`Paginator`] used in the query
    /// * `mined` - Whether to include transactions of type "Mined" in the response
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn list_transactions(
        &self,
        mined: bool,
        page: Option<&Paginator>,
    ) -> Result<TransactionPage, Error> {
        let query = Some(ListTransactionsQuery {
            exclude_mined: mined,
            page,
        });

        self.krist_get("/api/krist/transactions", query).await
    }

    /// Lists transactions in order from newest to oldest as a
    /// [`TransactionPage`]
    ///
    /// # Arguments
    /// * `page` -  The [`Paginator`] used in the query
    /// * `mined` - Whether to include transactions of type "Mined" in the response
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn new_transactions(
        &self,
        mined: bool,
        page: Option<&Paginator>,
    ) -> Result<TransactionPage, Error> {
        let query = Some(ListTransactionsQuery {
            exclude_mined: mined,
            page,
        });

        self.krist_get("/api/krist/transactions/latest", query)
            .await
    }

    /// Gets a specific [`Transaction`] by `id`. Will return None if the
    /// transaction does not exist
    ///
    /// # Errors
    /// Errors if there is a network issue
    ///
    /// See [`Error`] for more info
    pub async fn get_transaction(&self, id: u32) -> Result<Option<Transaction>, Error> {
        let url = format!("/api/krist/transactions/{id}");

        let res = self.krist_get::<TransactionRes>(&url, None::<()>).await;

        match res {
            Ok(tr) => Ok(Some(tr.transaction)),
            Err(Error::KristResponse {
                source: KristError::TransactionNotFound,
            }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Makes a Kromer [`Transaction`]. Note that this does preform several
    /// expensive hashes to convert a [`PrivateKey`] into an [`Address`] to
    /// ensure they are not the same as `addr`
    ///
    /// # Arguments
    /// * `addr` - The [`Address`] the transaction is going to
    /// * `amount` - The amount of Kromer to send
    /// * `meta` - The metadata to attach to this transaction
    /// * `pk` - The [`PrivateKey`] attached to the wallet sending the transaction
    ///
    /// # Errors
    /// Errors if both addresses are the same, or the wallet `pk` points to has
    /// insufficient funds.
    ///
    /// See [`Error`] for more info
    pub async fn make_transaction(
        &self,
        addr: &Address,
        amount: Decimal,
        meta: Option<&str>,
        pk: &PrivateKey,
    ) -> Result<Transaction, Error> {
        let pk_addr = Address::from(pk);

        ensure!(pk_addr != *addr, SameWalletTransferSnafu);

        let body = MakeTransactionBody {
            privatekey: pk,
            metadata: meta,
            to: addr,
            amount,
        };

        Ok(self
            .krist_post::<TransactionRes>("/api/krist/transactions", body)
            .await?
            .transaction)
    }
}
