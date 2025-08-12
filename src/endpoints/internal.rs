//! Endpoints targeting Kromer2's internal API. Available under the `internal` feature.
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    Privileged,
    endpoints::Endpoint,
    model::{
        Wallet,
        krist::{Address, PrivateKey},
    },
};

/// An endpoint for creating a new [`Address`]
#[derive(Debug, Serialize, Clone)]
pub struct CreateWalletEp {
    name: String,
    #[serde(rename = "uuid")]
    id: Uuid,
}

impl CreateWalletEp {
    /// Creates a new [`CreateWalletEp`]
    #[must_use]
    pub const fn new(name: String, id: Uuid) -> Self {
        Self { name, id }
    }
}

#[derive(Debug, Deserialize)]
struct CreateWalletRes {
    privatekey: PrivateKey,
    address: Address,
}

impl Endpoint<Privileged> for CreateWalletEp {
    type Value = (Address, PrivateKey);

    async fn query(
        &self,
        client: &crate::KromerClient<Privileged>,
    ) -> Result<Self::Value, crate::Error> {
        let res = client
            .internal_post::<CreateWalletRes>("/api/_internal/wallet/create", self)
            .await?;

        Ok((res.address, res.privatekey))
    }
}

/// An endpoint for adding money to an [`Address`]
#[derive(Debug, Serialize, Clone, Copy)]
pub struct GiveMoneyEp {
    #[serde(rename = "address")]
    addr: Address,
    amount: Decimal,
}

impl GiveMoneyEp {
    /// Creates a new [`GiveMoneyEp`]
    #[must_use]
    pub fn new(addr: Address, amount: impl Into<Decimal>) -> Self {
        Self {
            addr,
            amount: amount.into(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct InternalWalletRes {
    wallet: InternalWalletResInternal,
}

#[derive(Debug, Deserialize, Clone)]
struct InternalWalletResInternal {
    #[serde(flatten)]
    wallet: Wallet,
    #[serde(rename = "private_key")]
    pk: PrivateKey,
}

impl Endpoint<Privileged> for GiveMoneyEp {
    type Value = (Wallet, PrivateKey);

    async fn query(
        &self,
        client: &crate::KromerClient<Privileged>,
    ) -> Result<Self::Value, crate::Error> {
        let res = client
            .internal_post::<InternalWalletRes>("/api/_internal/wallet/give-money", self)
            .await?
            .wallet;

        Ok((res.wallet, res.pk))
    }
}

/// An endpoint for getting [`Wallets`](Wallet) by [`Uuid`].
/// Similar to the [`GetByUuidEp`](`super::GetByUuidEp`), but returns
/// the [`PrivateKey`] as well
#[derive(Debug, Clone, Copy)]
pub struct InternalByUuid {
    id: Uuid,
}

impl InternalByUuid {
    /// Creates a new [`InternalByUuid`]
    #[must_use]
    pub const fn new(id: Uuid) -> Self {
        Self { id }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct UuidListRes {
    wallet: Vec<InternalWalletResInternal>,
}

impl Endpoint<Privileged> for InternalByUuid {
    type Value = Vec<(Wallet, PrivateKey)>;

    async fn query(
        &self,
        client: &crate::KromerClient<Privileged>,
    ) -> Result<Self::Value, crate::Error> {
        let url = format!("/api/_internal/wallet/by-player/{}", self.id);

        let res = client.internal_get::<UuidListRes>(&url).await?.wallet;

        Ok(res.into_iter().map(|v| (v.wallet, v.pk)).collect())
    }
}
