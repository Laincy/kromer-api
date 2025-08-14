//! Type models for interacting with the Kromer2 API

pub use wallet::*;

pub mod krist;

mod wallet;

use serde::Deserialize;
use snafu::Snafu;

/// Errors returned by the Kromer2 API itself
#[derive(Debug, Snafu, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "snake_case")]
pub enum KromerError {
    /// 404 response
    #[snafu(display("Resource not found"))]
    ResourceNotFoundError,
    /// Kromer experienced an issue with wallets
    #[snafu(display("{message}"))]
    #[serde(rename = "wallet_error")]
    WalletError { message: String },
    /// Kromer experienced an issue handling the transaction
    #[snafu(display("{message}"))]
    TransactionError { message: String },
    /// Kromer experienced an issue with the player
    #[snafu(display("{message}"))]
    PlayerError { message: String },
    /// For errors within Kromer2 itself that can't be handled by us
    #[snafu(display("Kromer2 error: {message}"))]
    InternalServerError { message: String },
}

/// Error emitted when parsing objects in `kromer_2`
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum ParseError {
    /// Thrown when input exceeds the desired length
    #[snafu(display("exp {exp} bytes, got found {got}"))]
    UnexpectedLength { exp: u8, got: usize },
    /// Thrown when the input is not the special name `serverwelf` and doesn't
    /// start with a 'k
    #[snafu(display("expected bytes starting with 107 ('k'), found {got}"))]
    InvalidPrefix {
        /// The byte found
        got: u8,
    },
    /// Thrown when the input contains bytes that are not in the ranges 1-9 or
    /// a-z
    #[snafu(display(
        "expected a byte in ranges 46..=57 or 97..=122, found {got} at index {index} "
    ))]
    InvalidByte {
        /// The byte found
        got: u8,
        /// The index of the input at which the wrong byte was found
        index: usize,
    },
    /// Input string did not fall in the range `1..=64`
    #[snafu(display("Names must be between 1 and 64 characters long, found {len}"))]
    LengthBounds { len: usize },
    /// When the input string ends with an extension that is not `.kro`
    #[snafu(display(r#"Characters after '.' must be "kro""#))]
    BadSuffix,
    /// When the input contains invalid characters
    #[snafu(display("Names support alphanumeric characters, '-', and '_'. Found '{c}'"))]
    InvalidChar { c: char },
}
