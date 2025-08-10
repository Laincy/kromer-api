use crate::{
    endpoints::Endpoint,
    model::krist::{Address, Motd, PrivateKey},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// An endpoint for authenticating an [`Address`] using a [`PrivateKey`]
///
/// Returns an [`Option`] with `Some` if the wallet existed, and `None` if it did not.
///
/// See: <https://krist.dev/docs/#api-MiscellaneousGroup-Login>
#[derive(Debug, Serialize, Clone)]
pub struct AuthAddrEp {
    #[serde(rename = "privatekey")]
    pk: PrivateKey,
}

impl AuthAddrEp {
    /// Creates a new [`AuthAddrEp`]
    /// # Arguments
    /// * `pk` - The [`PrivateKey`] you would like to authenticate
    #[must_use]
    pub const fn new(pk: PrivateKey) -> Self {
        Self { pk }
    }
}

#[derive(Debug, Deserialize)]
struct AuthRes {
    address: Option<Address>,
}

impl Endpoint for AuthAddrEp {
    type Value = Option<Address>;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        Ok(client
            .post::<AuthRes>("/api/krist/login", self)
            .await?
            .address)
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

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        client.get("/api/krist/motd", None::<()>).await
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

#[derive(Debug, Deserialize)]
struct SupplyRes {
    money_supply: Decimal,
}

impl Endpoint for GetMoneySupplyEp {
    type Value = Decimal;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let res = client
            .get::<SupplyRes>("/api/krist/supply", None::<()>)
            .await?
            .money_supply;

        Ok(res)
    }
}
// We don't implament the MakeV2Address endpoint because we have our own internal methods for it
