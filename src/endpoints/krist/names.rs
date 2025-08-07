use crate::{
    KromerClient, KromerError,
    endpoints::{Endpoint, Paginated, PaginatedEndpoint},
    model::krist::{ExtractJson, GetCostRes, GetNameAvailRes, GetNameRes, Name, NamePage, PageRes},
};
use rust_decimal::Decimal;
use serde::Serialize;

/// An endpoint for getting information about a specific [`Name`]
///
/// See: <https://krist.dev/docs/#api-NameGroup-GetName>
#[derive(Debug, Clone)]
pub struct GetNameEp {
    name: String,
}

impl GetNameEp {
    /// Creates a new [`GetNameEp`]
    ///
    /// * `name` - The name you wish to fetch, without the .kro extension
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self { name }
    }
}

impl Endpoint for GetNameEp {
    type Value = Name;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let url = format!("/api/krist/names/{}", self.name);

        client.get::<GetNameRes>(&url, None::<()>).await?.extract()
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

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .get::<PageRes<Self::Value>>("/api/krist/names", Some(self))
            .await?
            .extract()
    }
}

impl PaginatedEndpoint for ListNamesEp {
    async fn query_page(&mut self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let res = self.query(client).await?;

        self.offset += res.count;

        Ok(res)
    }
}

/// An endpoint for listing the newest [`Names`](Name) as a [`NamePage`]
///
/// See: <https://krist.dev/docs/#api-NameGroup-GetNewNames>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct ListNewNamesEp {
    limit: usize,
    offset: usize,
}

impl ListNewNamesEp {
    /// Creates a new [`ListNewNamesEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

impl Default for ListNewNamesEp {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

impl Paginated for ListNewNamesEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for ListNewNamesEp {
    type Value = NamePage;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .get::<PageRes<Self::Value>>("/api/krist/names/new", Some(self))
            .await?
            .extract()
    }
}

impl PaginatedEndpoint for ListNewNamesEp {
    async fn query_page(&mut self, client: &KromerClient) -> Result<Self::Value, KromerError> {
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

impl Endpoint for GetNameCostEp {
    type Value = Decimal;

    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        client
            .get::<GetCostRes>("/api/krist/names/cost", None::<()>)
            .await?
            .extract()
    }
}

/// An endpoint for checking if a [`Name`] is available to purchase
///
/// See: <https://krist.dev/docs/#api-NameGroup-CheckName>
pub struct CheckNameAvailEp {
    name: String,
}

impl CheckNameAvailEp {
    /// Creates a new [`CheckNameAvailEp`]
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self { name }
    }
}

impl Endpoint for CheckNameAvailEp {
    type Value = bool;
    async fn query(&self, client: &KromerClient) -> Result<Self::Value, KromerError> {
        let url = format!("/api/krist/names/check/{}", self.name);

        client
            .get::<GetNameAvailRes>(&url, None::<()>)
            .await?
            .extract()
    }
}
