use crate::endpoints::{Endpoint, Paginated};
use crate::model::krist::{Address, PrivateKey, Transaction, TransactionPage};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::trace_span;

/// An endpoint for listing all transactions as a [`TransactionPage`]
///
/// See: <https://krist.dev/docs/#api-TransactionGroup-GetTransactions>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct ListTransactionsEp {
    #[serde(rename = "excludeMined")]
    query_mined: bool,
    limit: usize,
    offset: usize,
}

impl Default for ListTransactionsEp {
    fn default() -> Self {
        Self {
            query_mined: false,
            limit: 50,
            offset: 0,
        }
    }
}

impl ListTransactionsEp {
    /// Creates a new [`ListTransactionsEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            query_mined: false,
            limit: 50,
            offset: 0,
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

impl Paginated for ListTransactionsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for ListTransactionsEp {
    type Value = TransactionPage;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = trace_span!(
            "list_transactions",
            offset = self.offset,
            limit = self.limit
        );
        let _guard = span.enter();

        client.get("/api/krist/transactions", Some(self)).await
    }
}

/// An endpoint for listing the most recent transactions as a [`TransactionPage`]
///
/// See: <https://krist.dev/docs/#api-TransactionGroup-GetLatestTransactions>
#[derive(Debug, Serialize, Clone, Copy)]
pub struct LatestTransactionsEp {
    #[serde(rename = "excludeMined")]
    query_mined: bool,
    limit: usize,
    offset: usize,
}

impl Default for LatestTransactionsEp {
    fn default() -> Self {
        Self {
            query_mined: false,
            limit: 50,
            offset: 0,
        }
    }
}

impl LatestTransactionsEp {
    /// Creates a new [`LatestTransactionsEp`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            query_mined: false,
            limit: 50,
            offset: 0,
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

impl Paginated for LatestTransactionsEp {
    fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }
}

impl Endpoint for LatestTransactionsEp {
    type Value = TransactionPage;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = trace_span!(
            "recent_transactions",
            offset = self.offset,
            limit = self.limit
        );
        let _guard = span.enter();

        client
            .get("/api/krist/transactions/latest", Some(self))
            .await
    }
}

/// An endpoint for getting a specific `[Transaction]` by ID
///
/// See: <https://krist.dev/docs/#api-TransactionGroup-GetTransaction>
#[derive(Debug, Clone, Copy)]
pub struct GetTransactionEp {
    id: u32,
}

impl GetTransactionEp {
    /// Creates a new [`GetTransactionEp`]
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct TransactionRes {
    transaction: Transaction,
}

impl Endpoint for GetTransactionEp {
    type Value = Transaction;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = trace_span!("get_transaction", id = self.id);
        let _guard = span.enter();

        let url = format!("/api/krist/transactions/{}", self.id);

        Ok(client
            .get::<TransactionRes>(&url, None::<()>)
            .await?
            .transaction)
    }
}

/// An endpoint for making a transaction
///
/// See: <https://krist.dev/docs/#api-TransactionGroup-MakeTransaction>
#[derive(Debug, Serialize, Clone)]
pub struct MakeTransactionEp {
    #[serde(rename = "privatekey")]
    pk: PrivateKey,
    to: Address,
    amount: Decimal,
    metadata: Option<String>,
}

impl MakeTransactionEp {
    /// Creates a new [`MakeTransactionEp`]. Will return `None` if the address of `pk` and `addr`
    /// are the same. This is because you cannot make transactions between the same wallet. If you
    /// would like to avoid this check, use [`Self::new_unchecked`].
    ///
    /// # Arguments
    /// * `addr` - The [`Address`] to make the transaction with
    /// * `amount` - The amount of Kromer to send
    /// * `pk` - The [`PrivateKey`] of the account making the transaction
    #[must_use]
    pub fn new(addr: Address, amount: impl Into<Decimal>, pk: PrivateKey) -> Option<Self> {
        let my_addr: Address = pk.clone().into();

        if my_addr == addr {
            return None;
        }

        Some(Self {
            pk,
            amount: amount.into(),
            to: addr,
            metadata: None,
        })
    }

    /// Creates a new [`MakeTransactionEp`]. This method does *not* check if `addr` and the address
    /// of `pk` are the same, and may cause extra errors when querying the Kromer API. Only use
    /// this if you're ok with that, or sure that the address and pk are not the same. This avoids
    /// an expensive hashing operation. Otherwise, use [`Self::new`].
    ///
    /// # Arguments
    /// * `addr` - The [`Address`] to make the transaction with
    /// * `amount` - The amount of Kromer to send
    /// * `pk` - The [`PrivateKey`] of the account making the transaction
    #[must_use]
    pub fn new_unchecked(addr: Address, amount: impl Into<Decimal>, pk: PrivateKey) -> Self {
        Self {
            pk,
            amount: amount.into(),
            to: addr,
            metadata: None,
        }
    }

    /// Sets the metadata of the transaction
    #[must_use]
    pub fn set_meta(mut self, meta: String) -> Self {
        self.metadata = Some(meta);
        self
    }
}

impl Endpoint for MakeTransactionEp {
    type Value = Transaction;

    async fn query(&self, client: &crate::KromerClient) -> Result<Self::Value, crate::Error> {
        let span = trace_span!(
            "make_transaction",
            to = %self.to,
            amount = %self.amount,
            meta = ?self.metadata
        );
        let _guard = span.enter();

        Ok(client
            .post::<TransactionRes>("/api/krist/transactions", self)
            .await?
            .transaction)
    }
}
