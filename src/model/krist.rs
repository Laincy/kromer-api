//! Type models for the Krist compatability layer of the Kromer2 API

use crate::KromerError;
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{
    Deserialize, Serialize,
    de::{Deserializer, Error as DeError, Visitor},
};

/// Shared behavior for internal krist types
pub(crate) trait ExtractJson {
    /// The value we wish to extract
    type Res;

    /// Extracts a value from its deserialized JSON wrapper
    fn extract(self) -> Result<Self::Res, KromerError>;
}

/// A wallet address, stored as a null terminated array of up to 10 bytes
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct WalletAddr([u8; 10]);

impl WalletAddr {
    /// Serverwelf address
    pub const SERVERWELF: Self = Self(*b"serverwelf");

    /// Parses a slice of bytes into a `WalletId`.
    /// # Errors
    /// Will error if the ID is too long or contains non alphanumeric values
    pub const fn parse(value: &[u8]) -> Result<Self, WalletAddrParseError> {
        let len = value.len();

        if len > 10 {
            return Err(WalletAddrParseError::MaxLen(len));
        }

        let mut bytes = [0u8; 10];

        let mut i = 0;

        while i < len {
            let b = value[i];

            match value[i] {
                b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' => bytes[i] = b,
                _ => return Err(WalletAddrParseError::InvalidByte(b)),
            }

            i += 1;
        }

        Ok(Self(bytes))
    }

    #[must_use]
    /// Returns only the utilized bytes
    pub fn inner(&self) -> &[u8] {
        let mut len: usize = 0;

        while len < 10 {
            if self.0[len] == 0 {
                break;
            }
            len += 1;
        }

        &self.0[0..len]
    }
}
impl TryFrom<&str> for WalletAddr {
    type Error = WalletAddrParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value.as_bytes())
    }
}

impl std::fmt::Display for WalletAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            std::str::from_utf8(self.inner())
                .expect("Should never be wrong unless WalletAddr invariance is violated"),
        )
    }
}

impl<'de> Deserialize<'de> for WalletAddr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WalletAddrVisitor;

        impl Visitor<'_> for WalletAddrVisitor {
            type Value = WalletAddr;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("wallet address")
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                WalletAddr::parse(v.as_bytes()).map_err(DeError::custom)
            }
        }

        deserializer.deserialize_any(WalletAddrVisitor)
    }
}

impl Serialize for WalletAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Debug, thiserror::Error)]
/// Errors emmitted when creating a [`WalletAddr`]
pub enum WalletAddrParseError {
    /// Occurs when the passed value's length is longer than 10
    #[error("id exceeds maximum length. Must be less than 10, got {0}")]
    MaxLen(usize),
    /// Occurs when a non alphanumeric character is passed to the parse function
    #[error("input myst be alphanumeric, '-', or '_'. Got {0}")]
    InvalidByte(u8),
}

#[cfg(test)]
mod tests {
    use super::WalletId;

    #[test]
    fn parse() {
        assert_eq!(WalletId::parse(b"serverwel").unwrap().inner(), b"serverwel");
    }
}

/// A wallet fetched from the Kromer2 API
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Wallet {
    /// The address associated with this wallet
    #[allow(clippy::struct_field_names)]
    pub address: WalletAddr,
    /// The amount of Kromer owned by this address
    pub balance: Decimal,
    /// The total amount of Kromer that has ever gone into this address.
    #[serde(alias = "totalin")]
    pub total_in: Decimal,
    /// The total amount of Kromer that has ever gone out this address.
    #[serde(alias = "totalout")]
    pub total_out: Decimal,
    /// The date and time at which this wallet's first transaction was made.
    #[serde(alias = "firstseen")]
    pub first_seen: DateTime<Utc>,
    /// The numbeer of names owned by this wallet. Only present when using the `fetchNames`
    /// parameter under [`GetAddrEp`](`crate::endpoints::krist::GetAddrEp`)
    pub names: Option<u32>,
}

/// Internal deserialization type for the `/addresses/<addr>` endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum GetAddrRes {
    Address { address: Wallet },
    KristError { error: String, message: String },
}

impl ExtractJson for GetAddrRes {
    type Res = Wallet;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Address { address } => Ok(address),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}

/// A page of wallets fetched from a paginated API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletPage {
    /// The number of wallets returned in this query
    pub count: usize,
    /// The total number of wallets fetchable from the endpoint this value came from
    pub total: usize,
    /// The wallets fetched from this query
    #[serde(alias = "addresses")]
    pub wallets: Vec<Wallet>,
}

/// An intermediary result when using paginated endpoints
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum PageRes<T> {
    Page(T),
    KristError { error: String, message: String },
}

impl<T> ExtractJson for PageRes<T> {
    type Res = T;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Page(p) => Ok(p),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}

/// A Kromer2 transaction fetched from the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    /// The ID of this transaction
    pub id: u32,
    /// The address sending this transaction.
    pub from: Option<WalletAddr>,
    /// The address receiving this transaction. Will be `"name"` if the transaction was a name
    /// purchase, or `"a"` if it was a name's data change.
    pub to: WalletAddr,
    /// The amount of Kromer transferred in this transaction. Can be 0, notably if the transaction
    /// was a name's data change.
    pub value: Decimal,
    /// The date and time this transaction was made.
    pub time: DateTime<Utc>,
    /// The name associated with this transaction if there is one, without the `.kro` suffix.
    pub name: Option<String>,
    // TODO: Implement metadata parsing
    /// Transaction metadata
    pub metadata: Option<String>,
    /// The metaname (part before the `"@"`) of the recipiuent of the transaction, if it was sent to
    /// a name.
    pub sent_metaname: Option<String>,
    /// The name this transaction was sent to, without the `.kro` suffix, if it was sent to a name.
    pub sent_name: Option<String>,
    #[serde(alias = "type")]
    /// The type of this transaction.
    pub transaction_type: TransactionType,
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
    /// The total number of transactions that could ever be fetched by a query with the same
    /// parameters
    pub total: usize,
    /// The transactions fetched
    pub transactions: Vec<Transaction>,
}
