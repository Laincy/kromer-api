use serde::Serialize;
use uuid::Uuid;

use crate::{endpoints::Endpoint, model::Wallet};

/// An endpoint for getting all [`Wallets`](Wallet) owned by [`Uuid`]
#[derive(Debug, Serialize, Clone, Copy)]
pub struct GetByUuidEp {
    id: Uuid,
}

impl GetByUuidEp {
    /// Creates a new [`GetByUuidEp`]
    #[must_use]
    pub const fn new(id: Uuid) -> Self {
        Self { id }
    }
}

impl Endpoint for GetByUuidEp {
    type Value = Vec<Wallet>;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let url = format!("/api/v1/wallet/by-player/{}", self.id);
        client.get::<Vec<Wallet>>(&url).await
    }
}

/// An endpoint for getting all [`Wallets`](Wallet) owned by a Minecraft user
#[derive(Debug, Serialize, Clone)]
pub struct GetByUserEp {
    name: String,
}

impl GetByUserEp {
    /// Creates a new [`GetByUuidEp`]
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self { name }
    }
}

impl Endpoint for GetByUserEp {
    type Value = Vec<Wallet>;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let url = format!("/api/v1/wallet/by-name/{}", self.name);
        client.get::<Vec<Wallet>>(&url).await
    }
}
