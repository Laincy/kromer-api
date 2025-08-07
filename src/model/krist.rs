//! Type models for the Krist compatability layer of the Kromer2 API
//!

pub use wallet_addr::{WalletAddr, WalletAddrParseError};
pub use wallet_pk::{WalletPkParseError, WalletPrivateKey};

mod wallet_addr;
mod wallet_pk;

use crate::KromerError;
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Shared behavior for internal krist types
pub(crate) trait ExtractJson {
    /// The value we wish to extract
    type Res;

    /// Extracts a value from its deserialized JSON wrapper
    fn extract(self) -> Result<Self::Res, KromerError>;
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
    /// The total number of transactions that could ever be fetched from this endpoint
    pub total: usize,
    /// The transactions fetched
    pub transactions: Vec<Transaction>,
}

/// A name fetched from the Kromer2 API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Name {
    /// The name, without the `.kro` suffix
    pub name: String,
    /// The address that currently owns this name
    pub owner: WalletAddr,
    /// The address that originally purchased this name
    pub original_owner: Option<WalletAddr>,
    /// The date and time this name was registered
    pub registered: DateTime<Utc>,
    /// The date and time this name was last updated - eitheir the data changed, or it was transferred to a
    /// new owner
    pub updated: Option<DateTime<Utc>>,
    /// The date and time this name was last transferred to a new owner.
    pub transferred: Option<DateTime<Utc>>,
    /// The amount unpaid on the purchase of this name
    pub unpaid: Decimal,
}

/// A page of [`names`](Name) fetched from a paginated API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamePage {
    /// The number of names returned from this query
    pub count: usize,
    /// The total number of transaction that could ever be fetched from this endpoint
    pub total: usize,
    /// The names fetched
    pub names: Vec<Name>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum AuthAddrRes {
    Address { address: Option<WalletAddr> },
    KristError { error: String, message: String },
}

impl ExtractJson for AuthAddrRes {
    type Res = Option<WalletAddr>;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Address { address } => Ok(address),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// Message of the day. `Currency` field is ommitted since this doesn't change
pub struct Motd {
    // pub server_time: DateTime<Utc>,
    /// The message of the day
    #[serde(alias = "motd")]
    pub msg: String,
    /// The public URL associated with this server
    pub public_url: String,
    /// The websocket URL associated with this server
    pub public_ws_url: String,
    /// Whether transactions are currently available on the server
    pub transactions_enabled: bool,
    /// Whether the server is running in debug mode
    pub debug_mode: bool,
    /// Information about the server package currently running
    pub package: Package,
    /// An additional notice produced by the server
    pub notice: String,
}

/// The package section of the [Motd] struct
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Package {
    /// The name of the package
    pub name: String,
    /// The current version of the server in server
    pub version: String,
    /// The package authors
    pub author: String,
    // fucking euros
    /// The license the server is being used under
    #[serde(alias = "licence")]
    pub license: String,
    /// A link to the git repository of the server software
    pub repository: String,
    /// The git has of the currently running version of the server software
    pub git_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum MoneySupplyRes {
    Supply { money_supply: Decimal },
    KristError { error: String, message: String },
}

impl ExtractJson for MoneySupplyRes {
    type Res = Decimal;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Supply { money_supply } => Ok(money_supply),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum GetV2AddrRes {
    Address { address: WalletAddr },
    KristError { error: String, message: String },
}

impl ExtractJson for GetV2AddrRes {
    type Res = WalletAddr;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Address { address } => Ok(address),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum GetNameRes {
    Name { name: Name },
    KristError { error: String, message: String },
}

impl ExtractJson for GetNameRes {
    type Res = Name;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Name { name } => Ok(name),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum GetCostRes {
    Cost { name_cost: Decimal },
    KristError { error: String, message: String },
}

impl ExtractJson for GetCostRes {
    type Res = Decimal;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Cost { name_cost } => Ok(name_cost),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum GetNameAvailRes {
    Avail { available: bool },
    KristError { error: String, message: String },
}

impl ExtractJson for GetNameAvailRes {
    type Res = bool;

    fn extract(self) -> Result<Self::Res, KromerError> {
        match self {
            Self::Avail { available } => Ok(available),
            Self::KristError { error, message } => Err(KromerError::Krist { error, message }),
        }
    }
}
