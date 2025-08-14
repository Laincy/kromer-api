//! Types modelling the Krist compatible section of the Kromer2 API

use super::Wallet;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::fmt::Debug;

pub use names::*;
pub use transactions::*;

mod names;
mod transactions;

/// Errors that can be emmitted by the Krist API
#[derive(Debug, Snafu, PartialEq, Eq)]
#[allow(missing_docs)]
#[snafu(visibility(pub(crate)))]
pub enum KristError {
    // We skip generic and path errors since they (in theory) shouldn't occur. Might change this to
    // pass them on later as well.
    //
    // I will probably implement websocket errors as a seperate type, since they shouldn't conflict
    // *too* often
    #[snafu(display(r#"Address "{addr}" could not be found"#))]
    AddrNotFound {
        // String used here instead of address so that we can still bubble up the returned value
        // even if it's not a valid address. It should always be since we'll only submit valid
        // addresses, but still
        addr: String,
    },
    #[snafu(display("Authentication failed"))]
    AuthFailed,
    #[snafu(display(r#"Could't find name "{name}""#))]
    NameNotFound { name: String },
    #[snafu(display(r#"Name "{name}" is already taken "#))]
    NameTaken { name: String },
    #[snafu(display(r#"Client is not authorized to modify name "{name}""#))]
    NotNameOwner { name: String },
    // TODO: Make sure that the `InsufficientFunds` error also maps to this
    #[snafu(display("Insufficent balance"))]
    InsufficientBalance,
    #[snafu(display("Could not find transaction"))]
    TransactionNotFound,
    #[snafu(display("Trasactions are disabled on this server"))]
    TransactionsDisabled,
    // TODO
    /// This library *should* prevent this, but it's here anyways
    #[snafu(display("Attempted to transfer into the same wallet"))]
    SameWalletTransfer,
    #[snafu(display(r#"Transaction conflict for parameter "{param}""#))]
    TransactionConflict { param: String },
    /// Various internal errors are exposed under the same name in the `error` field of the JSON
    /// response, but have different messages. We just pass the message up
    /// much we're able to to about it.
    #[snafu(display("Kromer2 server error: {message}"))]
    InternalServerError { message: String },
    #[snafu(display("Recieved an unexpected response"))]
    UnexpectedResponse,
}

/// Message of the day. `Currency` field is ommitted since this doesn't change
#[derive(Debug, Deserialize, Serialize, Clone)]
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

/// A page of wallets fetched from the Krist API
#[derive(Debug, Deserialize, Clone)]
pub struct WalletPage {
    /// The wallets fetched
    #[serde(rename = "addresses")]
    pub wallets: Vec<Wallet>,
    /// The number of wallets recieved
    pub count: usize,
    /// The total wallets that can be fetched
    pub total: usize,
}
