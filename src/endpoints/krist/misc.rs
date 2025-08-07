use crate::{
    KromerClient, KromerError,
    endpoints::Endpoint,
    model::krist::{
        AuthAddrRes, ExtractJson, GetV2AddrRes, MoneySupplyRes, Motd, WalletAddr, WalletPrivateKey,
    },
};
use rust_decimal::Decimal;
use serde::Serialize;

/// An endpoint for authenticating a [`WalletAddr`] using a [`WalletPrivateKey`]
///
/// Returns an [`Option`] with `Some` if the wallet existed, and `None` if it did not.
///
/// See: <https://krist.dev/docs/#api-MiscellaneousGroup-Login>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct AuthAddrEp {
    #[serde(rename = "privatekey")]
    pk: WalletPrivateKey,
}

impl AuthAddrEp {
    /// Creates a new [`AuthAddrEp`]
    #[must_use]
    pub const fn new(pk: WalletPrivateKey) -> Self {
        Self { pk }
    }
}

impl Endpoint for AuthAddrEp {
    type Value = Option<WalletAddr>;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .post::<AuthAddrRes>("/api/krist/login", self)
            .await?
            .extract()
    }
}

/// An endpoint for getting the [`Motd`]
///
/// See: <https://krist.dev/docs/#api-MiscellaneousGroup-GetMOTD>
#[derive(Debug, Default, Clone, Copy)]
pub struct GetMotdEp;

impl GetMotdEp {
    /// Creates a new [`GetMotdEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Endpoint for GetMotdEp {
    type Value = Motd;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        // We directly query here since there should be no chance of the server throwing an error
        // unless something is fucked on their side, in which case we don't need to care about Motd
        // anyways

        Ok(client
            .http
            .get(client.url.join("/api/krist/motd").expect("never fail"))
            .send()
            .await?
            .error_for_status()?
            .json::<Self::Value>()
            .await?)
    }
}

/// An endpoint for getting the amount of money in circulation
///
/// See: <https://krist.dev/docs/#api-MiscellaneousGroup-GetMoneySupply>
#[derive(Debug, Default, Clone, Copy)]
pub struct GetMoneySupplyEp;

impl GetMoneySupplyEp {
    /// Creates a new [`GetMoneySupplyEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Endpoint for GetMoneySupplyEp {
    type Value = Decimal;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .get::<MoneySupplyRes>("/api/krist/supply", None::<()>)
            .await?
            .extract()
    }
}

/// An endpoint for turning a [`WalletPrivateKey`] into a [`WalletAddr`]
///
/// See: <https://krist.dev/docs/#api-MiscellaneousGroup-MakeV2Address>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct GetV2FromPkEp {
    #[serde(rename = "privatekey")]
    pk: WalletPrivateKey,
}

impl GetV2FromPkEp {
    /// Creates a new [`GetV2FromPkEp`]
    #[must_use]
    pub const fn new(pk: WalletPrivateKey) -> Self {
        Self { pk }
    }
}

impl Endpoint for GetV2FromPkEp {
    type Value = WalletAddr;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .post::<GetV2AddrRes>("/api/krist/v2", self)
            .await?
            .extract()
    }
}
