//! Type models for interacting with the Kromer2 API

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::model::krist::Address;

pub mod krist;

/// Errors emmitted by the Kromer specific API. This enum is much less verbose than the
/// [`KristError`](krist::KristError) error as Kromer errors emit much less information about the
/// underlying errors.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum KromerError {
    /// 404 response
    #[snafu(display("Resource not found"))]
    NotFound,
    /// Kromer experienced an issue with wallets
    #[snafu(display("{message}"))]
    Wallet { message: String },
    /// Kromer experienced an issue handling the transaction
    #[snafu(display("{message}"))]
    Transaction { message: String },
    /// Kromer experienced an issue with the player
    #[snafu(display("{message}"))]
    Player { message: String },
    #[snafu(display("Kromer2 server error: {message}"))]
    InternalServerError { message: String },
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawKromerError {
    code: String,
    message: String,
}

impl RawKromerError {
    pub fn parse(self) -> Result<(), KromerError> {
        Err(match self.code.as_str() {
            "resource_not_found_error" => KromerError::NotFound,
            "wallet_error" => KromerError::Wallet {
                message: self.message,
            },
            "transaction_error" => KromerError::Transaction {
                message: self.message,
            },
            "player_error" => KromerError::Player {
                message: self.message,
            },
            _ => KromerError::InternalServerError {
                message: self.message,
            },
        })
    }
}

/// A wallet fetched from the Kromer2 API
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Wallet {
    /// The internal ID of the wallet
    pub id: u32,
    /// The [`Address`] associated with the wallet
    pub address: Address,
    /// The amount of Kromer in this wallet
    pub balance: Decimal,
    /// When this wallet was created
    pub created_at: DateTime<Utc>,
    /// Whether this wallet can make transactions
    pub locked: bool,
    /// The total amount of Kromer that has been sent to this wallet
    pub total_in: Decimal,
    /// The total amount of Kromet that has been sent from this wallet
    pub total_out: Decimal,
}
