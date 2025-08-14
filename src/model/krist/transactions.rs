use super::Name;
use crate::model::Address;
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserializer;
use serde::{Deserialize, Serialize};

/// A Kromer2 transaction fetched from the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    /// The ID of this transaction
    pub id: u32,
    /// The address sending this transaction.
    pub from: Option<Address>,
    /// The address receiving this transaction. Will be `"name"` if the
    /// transaction was a name purchase, or `"a"` if it was a name's data
    /// change.
    pub to: Address,
    /// The amount of Kromer transferred in this transaction. Can be 0, notably
    /// if the transaction was a name's data change.
    pub value: Decimal,
    /// The date and time this transaction was made.
    pub time: DateTime<Utc>,
    /// The name associated with this transaction if there is one, without the
    /// `.kro` suffix.
    pub name: Option<String>,
    // TODO: Implement metadata parsing
    /// Transaction metadata
    #[serde(deserialize_with = "empty_string_is_none")]
    pub metadata: Option<String>,
    /// The `metaname` (part before the `"@"`) of the recipient of the
    /// transaction, if it was sent to a name.
    pub sent_metaname: Option<String>,
    /// The name this transaction was sent to, without the `.kro` suffix, if it
    /// was sent to a name.
    pub sent_name: Option<Name>,
    #[serde(alias = "type")]
    /// The type of this transaction.
    pub transaction_type: TransactionType,
}

fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
}

/// The type of a [`Transaction`]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum TransactionType {
    Mined,
    Transfer,
    NamePurchase,
    NameARecord,
    NameTransfer,
}

/// A page of [`transactions`](Transaction) fetched from a paginated API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionPage {
    /// The number of transactions returned from this query
    pub count: usize,
    /// The total number of transactions that could ever be fetched from this
    /// endpoint
    pub total: usize,
    /// The transactions fetched
    pub transactions: Vec<Transaction>,
}
