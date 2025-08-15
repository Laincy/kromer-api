//! Type models for the Kromer websocket API

use crate::model::krist::{NameInfo, Transaction};
use serde::{Deserialize, Serialize};

/// An event received over websocket
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "event", rename_all = "camelCase")]
#[allow(missing_docs)]
pub enum WebSocketEvent {
    Transaction { transaction: Transaction },
    Name { name: NameInfo },
}

/// Event types a client can subscribe to
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionType {
    /// All transactions on the server
    Transactions,
    /// All transactions involving the currently authorized address
    OwnTransactions,
    /// All name changes
    Names,
    /// All name changes involving the currently authorized address
    OwnNames,
    /// Not relevant in Kromer2, while this is an option to subscribe to you will nerver receive any events
    /// from here. It is only included for deserialization purposes
    Blocks,
}
