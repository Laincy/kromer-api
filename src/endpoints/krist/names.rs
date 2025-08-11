use crate::{
    endpoints::{Endpoint, Paginated, PaginatedEndpoint},
    model::krist::{Address, Name, NameInfo, NamePage, PrivateKey},
};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{Level, span};

/// An endpoint for getting information about a specific [`Name`]
///
/// See: <https://krist.dev/docs/#api-NameGroup-GetName>
#[derive(Debug, Clone)]
pub struct GetNameEp {
    name: Name,
}

impl GetNameEp {
    /// Creates a new [`GetNameEp`]
    /// # Arguments
    /// * `name` - The [`Name`] to fetch
    #[must_use]
    pub const fn new(name: Name) -> Self {
        Self { name }
    }
}

#[derive(Debug, Deserialize)]
struct NameRes {
    name: NameInfo,
}

impl Endpoint for GetNameEp {
    type Value = NameInfo;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(Level::TRACE, "GetName", name = %self.name);
        let _guard = span.enter();

        let url = format!("/api/krist/names/{}", self.name);

        Ok(client.krist_get::<NameRes>(&url, None::<()>).await?.name)
    }
}

/// An endpoint for listing all [`Names`](Name) as a [`NamePage`]
///
/// See: <https://krist.dev/docs/#api-NameGroup-GetNames>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct ListNamesEp {
    limit: usize,
    offset: usize,
}

impl ListNamesEp {
    /// Creates a new [`ListNamesEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

impl Default for ListNamesEp {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

impl Paginated for ListNamesEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for ListNamesEp {
    type Value = NamePage;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(
            Level::TRACE,
            "list_names",
            offset = self.offset,
            limit = self.limit
        );
        let _guard = span.enter();

        client.krist_get("/api/krist/names", Some(self)).await
    }
}

impl PaginatedEndpoint for ListNamesEp {
    async fn query_page(
        &mut self,
        client: &crate::KromerClient,
    ) -> Result<Self::Value, crate::Error> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}

/// An endpoint for getting the cost to buy a [`Name`]
///
/// See: <https://krist.dev/docs/#api-NameGroup-GetNameCost>
#[derive(Debug, Default, Clone, Copy)]
pub struct GetNameCostEp;

impl GetNameCostEp {
    /// Creates a new [`GetNameCostEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct CostRes {
    name_cost: Decimal,
}

impl Endpoint for GetNameCostEp {
    type Value = Decimal;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(Level::TRACE, "get_name_cost");
        let _guard = span.enter();

        Ok(client
            .krist_get::<CostRes>("/api/krist/names/cost", None::<()>)
            .await?
            .name_cost)
    }
}

/// An endpoint for checking if a [`Name`] is available to purchase
///
/// See: <https://krist.dev/docs/#api-NameGroup-CheckName>
#[derive(Debug, Clone)]
pub struct CheckNameAvailEp {
    name: Name,
}

impl CheckNameAvailEp {
    /// Creates a new [`CheckNameAvailEp`]
    /// # Arguments
    /// * `name` - The [`Name`] you would like to check
    #[must_use]
    pub const fn new(name: Name) -> Self {
        Self { name }
    }
}

#[derive(Debug, Deserialize)]
struct AvailRes {
    available: bool,
}

impl Endpoint for CheckNameAvailEp {
    type Value = bool;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(Level::TRACE, "check_name_avail", name = %self.name);
        let _guard = span.enter();

        let url = format!("/api/krist/names/check/{}", self.name);
        Ok(client.krist_get::<AvailRes>(&url, None::<()>).await?.available)
    }
}

// TODO: Test this shit on my own server since I'm too broke to do it on RCC
/// An endpoint for registering a new [`Name`].
///
/// See: <https://krist.dev/docs/#api-NameGroup-RegisterName>
#[derive(Debug, Serialize, Clone)]
pub struct RegisterNameEp {
    #[serde(skip)]
    name: Name,
    #[serde(rename = "privatekey")]
    pk: PrivateKey,
}

impl RegisterNameEp {
    /// Creates a new [`RegisterNameEp`]
    /// # Arguments
    /// * `name`: The [`Name`] you would like to register
    /// * `pk`: The [`PrivateKey`] of the address that the will pay for and own the name
    #[must_use]
    pub const fn new(name: Name, pk: PrivateKey) -> Self {
        Self { name, pk }
    }
}

impl Endpoint for RegisterNameEp {
    type Value = ();

    /// UNTESTED FUNCTION BECAUSE I'M BROKE AND CAN'T BE BOTHERED TO MAKE A PROPER TEST RIG
    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(Level::TRACE, "register_name", name = %self.name);
        let _guard = span.enter();

        let url = format!("/api/krist/names/{}", self.name);
        client.krist_post::<()>(&url, self).await
    }
}

/// An endpoint for transferring a [`Name`] to a new [`Address`]
///
/// See: <https://krist.dev/docs/#api-NameGroup-TransferName>
#[derive(Debug, Serialize, Clone)]
pub struct TransferNameEp {
    #[serde(skip)]
    name: Name,
    #[serde(rename = "address")]
    addr: Address,
    #[serde(rename = "privatekey")]
    pk: PrivateKey,
}

impl TransferNameEp {
    /// Creates a new [`TransferNameEp`]
    /// # Arguments
    /// * `name` - The [`Name`] that you want to transfer
    /// * `addr` - The [`Address`] that you would like to transfer the name to
    /// * `pk` - The [`PrivateKey`] of the address that currently owns the name
    #[must_use]
    pub const fn new(name: Name, addr: Address, pk: PrivateKey) -> Self {
        Self { name, addr, pk }
    }
}

impl Endpoint for TransferNameEp {
    type Value = NameInfo;

    /// UNTESTED FUNCTION BECAUSE I'M BROKE AND CAN'T BE BOTHERED TO MAKE A PROPER TEST RIG
    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(Level::TRACE, "transfer_name", addr = %self.addr, name = %self.name);
        let _guard = span.enter();
        let url = format!("/api/krist/names/{}/transfer", self.name);

        Ok(client.krist_post::<NameRes>(&url, self).await?.name)
    }
}

/// Endpoint for updating the metadata of a [`Name`].
///
/// While this endpoint uses the `POST` method,
/// it does the exact same thiing that the redundant Krist `PUT` endpoint would as well.
///
/// See: <https://krist.dev/docs/#api-NameGroup-UpdateNamePOST>
#[derive(Debug, Serialize, Clone)]
pub struct UpdateNameEp {
    #[serde(skip)]
    name: Name,
    #[serde(rename = "privatekey")]
    pk: PrivateKey,
    a: Option<String>,
}

impl UpdateNameEp {
    /// Creates a new [`UpdateNameEp`]
    /// # Arguments
    /// * `name` - The [`Name`] to update
    /// * `meta` - The metadata to set. Leaving this as `None` will remove all data
    #[must_use]
    pub fn new(name: Name, meta: Option<String>, pk: PrivateKey) -> Self {
        let a = meta.filter(String::is_empty);

        Self { name, pk, a }
    }
}

impl Endpoint for UpdateNameEp {
    type Value = NameInfo;

    /// UNTESTED FUNCTION BECAUSE I'M BROKE AND CAN'T BE BOTHERED TO MAKE A PROPER TEST RIG
    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = span!(Level::TRACE, "update_name", name = %self.name);
        let _guard = span.enter();

        let url = format!("/api/krist/names/{}/update", self.name);

        Ok(client.krist_post::<NameRes>(&url, self).await?.name)
    }
}
