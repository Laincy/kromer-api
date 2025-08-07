use crate::{
    KromerClient, KromerError,
    endpoints::Endpoint,
    model::krist::{AuthAddrRes, ExtractJson, Motd, WalletAddr, WalletPrivateKey},
};
use async_trait::async_trait;
use serde::Serialize;

/// An endpoint for authenticating a [`WalletAddr`] using a [`WalletPrivateKey`]
///
/// Returns an [`Option`] with `Some` if the wallet existed, and `None` if it did not.
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

#[async_trait]
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
#[derive(Debug, Default, Clone, Copy)]
pub struct GetMotdEp;

impl GetMotdEp {
    /// Creates a new [`GetMotdEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
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
