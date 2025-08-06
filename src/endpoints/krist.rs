/*! Endpoints that deal with the Krist compatability API as defined in
    the [Krist docs](https://krist.dev/docs/). Each struct handles
    interactions with a specific endpoint defined in the docs, alongside
    the [`Endpoint`], [`Paginated`] and [`PaginatedEndpoint`] traits to
    set parameters and query the API.
*/

use crate::{
    KromerClient, KromerError,
    endpoints::{Endpoint, Paginated},
    model::krist::{
        ExtractJson, GetAddrRes, PageRes, TransactionPage, Wallet, WalletAddr, WalletPage,
    },
};
use async_trait::async_trait;
use serde::Serialize;

use super::PaginatedEndpoint;

/// An endpoint for fetching a [`Wallet`] by [`WalletAddr`]
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddress>
#[derive(Debug, Serialize)]
pub struct GetAddrEp {
    #[serde(skip_serializing)]
    addr: WalletAddr,
    #[serde(alias = "fetchNames")]
    query_names: bool,
}

impl GetAddrEp {
    /// Creates a new [`GetAddrEp`]
    #[must_use]
    pub const fn new(addr: WalletAddr) -> Self {
        Self {
            addr,
            query_names: false,
        }
    }

    #[must_use]
    /// Sets the fetchNames query parameter
    pub const fn fetch_names(mut self, v: bool) -> Self {
        self.query_names = v;
        self
    }
}

#[async_trait]
impl Endpoint for GetAddrEp {
    type Value = Wallet;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let url = format!("/api/krist/addresses/{}", self.addr);

        client.get::<GetAddrRes>(&url, Some(&self)).await?.extract()
    }
}

/// An endpoint for listing [`Wallets`](Wallet) as a [`WalletPage`]
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddreses>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct ListAddrsEp {
    offset: usize,
    limit: usize,
}

impl ListAddrsEp {
    /// Creates a new [`ListAddrsEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl Default for ListAddrsEp {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl Paginated for ListAddrsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

#[async_trait]
impl Endpoint for ListAddrsEp {
    type Value = WalletPage;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .get::<PageRes<Self::Value>>("/api/krist/addresses", Some(self))
            .await?
            .extract()
    }
}

#[async_trait]
impl PaginatedEndpoint for ListAddrsEp {
    async fn query_page(&mut self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}

/// An endpoint for fetching the richest [`Wallets`](Wallet) as a [`WalletPage`]
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddreses>
#[derive(Debug, Serialize)]
pub struct RichAddrsEp {
    offset: usize,
    limit: usize,
}

impl RichAddrsEp {
    /// Creates a new [`RichAddrsEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl Default for RichAddrsEp {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl Paginated for RichAddrsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

#[async_trait]
impl Endpoint for RichAddrsEp {
    type Value = WalletPage;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .get::<PageRes<Self::Value>>("/api/krist/addresses/rich", Some(self))
            .await?
            .extract()
    }
}

#[async_trait]
impl PaginatedEndpoint for RichAddrsEp {
    async fn query_page(&mut self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}

/// An endpoint for fetching recent [`Transactions`](crate::model::krist::Transaction) of a given [`address`](WalletAddr)
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddressTransactions>
#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct RecentAddrTransactionsEp {
    #[serde()]
    addr: WalletAddr,
    query_mined: bool,
    limit: usize,
    offset: usize,
}
impl RecentAddrTransactionsEp {
    /// Creates a new [`RecentAddrTransactionsEp`]
    #[must_use]
    pub const fn new(addr: WalletAddr) -> Self {
        Self {
            addr,
            query_mined: false,
            limit: 50,
            offset: 0,
        }
    }

    /// Sets the `excludeMined` parameter on the query. This determines if the returned values will
    /// exclude transactions of the [`Mined`] type. Defaults to false if unset.
    ///
    /// [`Mined`]: crate::model::krist::TransactionType.Mined
    #[must_use]
    pub const fn exclude_mined(mut self, v: bool) -> Self {
        self.query_mined = v;
        self
    }

    /// Sets the targeted `address` to `addr`
    #[must_use]
    pub const fn set_address(mut self, addr: WalletAddr) -> Self {
        self.addr = addr;
        self
    }
}

impl Paginated for RecentAddrTransactionsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

#[async_trait]
impl Endpoint for RecentAddrTransactionsEp {
    type Value = TransactionPage;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let url = format!("/api/krist/addresses/{}/transactions", self.addr);

        client
            .get::<PageRes<Self::Value>>(&url, Some(self))
            .await?
            .extract()
    }
}

#[async_trait]
impl PaginatedEndpoint for RecentAddrTransactionsEp {
    async fn query_page(&mut self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}
