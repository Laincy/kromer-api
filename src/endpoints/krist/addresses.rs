use crate::{
    endpoints::{Endpoint, Paginated, PaginatedEndpoint},
    model::krist::{Address, NamePage, TransactionPage, Wallet, WalletPage},
};
use serde::{Deserialize, Serialize};
use tracing::{Level, span};

/// An endpoint for fetching a [`Wallet`] using an [`Address`]
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddress>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct GetWalletEp {
    #[serde(skip)]
    addr: Address,
    #[serde(rename = "fetchNames")]
    query_names: bool,
}

impl GetWalletEp {
    /// Creates a new [`GetWalletEp`]
    #[must_use]
    pub const fn new(addr: Address) -> Self {
        Self {
            addr,
            query_names: false,
        }
    }

    /// Sets the `fetchNames` parameter.
    #[must_use]
    pub const fn fetch_names(mut self, b: bool) -> Self {
        self.query_names = b;
        self
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct GetWalletRes {
    address: GetWalletResInner,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetWalletResInner {
    #[serde(flatten)]
    wallet: Wallet,
    names: Option<u32>,
}

impl Endpoint for GetWalletEp {
    type Value = (Wallet, Option<u32>);

    /// Gets the desired value from the API. Returns a tuple where the
    /// first value is the [`Wallet`] and the second is the optional
    /// names field, containing the number of names a wallet owns.
    /// This field will only be [`Some`] in the event that the `fetchNames`
    /// parameter was set to true. Otherwise it can be safely ignored.
    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(Level::TRACE, "get_walet", address = %self.addr);
        let _guard = span.enter();

        let url = format!("/api/krist/addresses/{}", self.addr);

        let res = client.get::<GetWalletRes>(&url, Some(self)).await?.address;

        Ok((res.wallet, res.names))
    }
}

/// An endpoint for listing [`Wallets`](Wallet) as a [`WalletPage`]
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddresses>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct ListWalletsEp {
    offset: usize,
    limit: usize,
}

impl Default for ListWalletsEp {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl ListWalletsEp {
    /// Creates a new [`ListWalletsEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl Paginated for ListWalletsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for ListWalletsEp {
    type Value = WalletPage;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(
            Level::TRACE,
            "list_wallets",
            limit = self.limit,
            offset = self.offset
        );
        let _guard = span.enter();
        client.get("/api/krist/addresses", Some(self)).await
    }
}

impl PaginatedEndpoint for ListWalletsEp {
    async fn query_page(
        &mut self,
        client: &crate::KromerClient,
    ) -> Result<Self::Value, crate::Error> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}

/// An endpoint for fetching the richest [`Wallets`](Wallet) as a [`WalletPage`]
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetRichAddresses>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct RichWalletsEp {
    offset: usize,
    limit: usize,
}

impl Default for RichWalletsEp {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl RichWalletsEp {
    /// Creates a new [`RichWalletsEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl Paginated for RichWalletsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for RichWalletsEp {
    type Value = WalletPage;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(
            Level::TRACE,
            "list_rich_wallets",
            limit = self.limit,
            offset = self.offset
        );
        let _guard = span.enter();

        client.get("/api/krist/addresses", Some(self)).await
    }
}

/// An endpoint for listing the most recent transactions at a given [`Address`]
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddressTransactions>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct RecentWalletTransactionsEp {
    #[serde(skip)]
    addr: Address,
    #[serde(rename = "excludeMined")]
    query_mined: bool,
    offset: usize,
    limit: usize,
}

impl RecentWalletTransactionsEp {
    /// Creates a new [`RecentWalletTransactionsEp`]
    #[must_use]
    pub const fn new(addr: Address) -> Self {
        Self {
            addr,
            query_mined: false,
            offset: 0,
            limit: 50,
        }
    }

    /// Sets the `excludeMined` query parameter which controls whether
    /// mined transactions are included in the response. Defaults to false
    #[must_use]
    pub const fn exclude_mined(mut self, b: bool) -> Self {
        self.query_mined = b;
        self
    }
}

impl Paginated for RecentWalletTransactionsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for RecentWalletTransactionsEp {
    type Value = TransactionPage;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(
            Level::TRACE,
            "recent_wallet_transactions",
            addr = %self.addr,
            limit = self.limit,
            offset = self.offset
        );
        let _guard = span.enter();

        let url = format!("/api/krist/addresses/{}/transactions", self.addr);
        client.get(&url, Some(self)).await
    }
}

impl PaginatedEndpoint for RecentWalletTransactionsEp {
    async fn query_page(
        &mut self,
        client: &crate::KromerClient,
    ) -> Result<Self::Value, crate::Error> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}

/// An endpioint for fetching all the [`Names`](crate::model::krist::Name) owned by a specific [`Address`].
///
/// See: <https://krist.dev/docs/#api-AddressGroup-GetAddressNames>
#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct ListWalletNamesEp {
    #[serde(skip)]
    addr: Address,
    limit: usize,
    offset: usize,
}

impl ListWalletNamesEp {
    /// Creates a new [`ListWalletNamesEp`]
    #[must_use]
    pub const fn new(addr: Address) -> Self {
        Self {
            addr,
            limit: 50,
            offset: 0,
        }
    }
}

impl Paginated for ListWalletNamesEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for ListWalletNamesEp {
    type Value = NamePage;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(
            Level::TRACE,
            "list_wallet_names",
            addr = %self.addr,
            limit = self.limit,
            offset = self.offset
        );
        let _guard = span.enter();

        let url = format!("/api/krist/addresses/{}/names", self.addr);
        client.get(&url, Some(self)).await
    }
}

impl PaginatedEndpoint for ListWalletNamesEp {
    async fn query_page(
        &mut self,
        client: &crate::KromerClient,
    ) -> Result<Self::Value, crate::Error> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}
