use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::model::{
    Address, Wallet,
    krist::Transaction,
    ws::{SubscriptionType, WebSocketEvent},
};

/// A request sent to Kromer2 via websocket
#[derive(Debug, Serialize, Clone)]
pub struct WebSocketRequest {
    pub id: usize,
    #[serde(flatten)]
    pub inner: WebSocketRequestInner,
}

impl WebSocketRequest {
    pub(crate) fn into_message(self) -> Message {
        #[allow(clippy::expect_used)]
        let req = serde_json::to_string(&self).expect("In shambles");
        Message::text(req)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebSocketMessage {
    pub(crate) id: Option<usize>,
    #[serde(flatten)]
    pub(crate) msg: WebSocketMessageInner,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WebSocketMessageInner {
    Event {
        #[serde(flatten)]
        event: WebSocketEvent,
    },
    Error {
        error: String,
        message: String,
    },
    KeepAlive,

    Response {
        #[serde(flatten)]
        // #[serde(with = "::serde_with::rust::maps_first_key_wins")]
        responding_to: MessageResponseInner,
    },
    Hello,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "responding_to", rename_all = "snake_case")]
pub enum MessageResponseInner {
    Address {
        address: Wallet,
    },
    Me {
        is_guest: bool,
        address: Option<Wallet>,
    },
    MakeTransaction {
        transaction: Transaction,
    },
    Logout {
        is_guest: bool,
    },

    Login {
        is_guest: bool,
        address: Option<Wallet>,
    },

    /// Used for both subscribe and unsubscribe responses
    Subscribe {
        subscription_level: Vec<SubscriptionType>,
    },
    GetSubscriptionLevel {
        subscription_level: Vec<SubscriptionType>,
    },
    GetValidSubscriptionLevels {
        valid_subscription_level: Vec<String>,
    },
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WebSocketRequestInner {
    Subscribe { event: SubscriptionType },
    Unsubscribe { event: SubscriptionType },
    Address { address: Address },
    GetSubscriptionLevel,
}
